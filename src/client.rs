use crate::protocol::inbound::{
    BroadcastingEvent, EntrylistCar, EntrylistUpdate, InboundMessage, RealtimeCarUpdate,
    RealtimeUpdate, TrackData,
};
use crate::protocol::outbound::{OutboundMessage, RegistrationRequest, UnregisterRequest};
use crate::session::Context;
use log::{debug, info, trace};
use nom_supreme::error::ErrorTree;
use nom_supreme::final_parser::ByteOffset;
use std::net::{ToSocketAddrs, UdpSocket};
use std::time::Duration;
use thiserror::Error;

const UDP_MAX: usize = 65535;

pub trait MessageHandler {
    fn realtime_update<M: MessageHandler>(
        &self,
        _client: &BroadcastingClient<M>,
        update: &RealtimeUpdate,
    ) {
        trace!(
            "Received realtime update packet for time {}",
            update.session_time
        );
    }

    fn realtime_car_update<M: MessageHandler>(
        &self,
        _client: &BroadcastingClient<M>,
        update: &RealtimeCarUpdate,
    ) {
        trace!("Received realtime car update for car ID {}", update.id);
    }

    fn entrylist_update<M: MessageHandler>(
        &self,
        _client: &BroadcastingClient<M>,
        update: &EntrylistUpdate,
    ) {
        debug!(
            "Received entry list update with {} cars",
            update.car_ids.len()
        );
    }

    fn entrylist_car<M: MessageHandler>(
        &self,
        _client: &BroadcastingClient<M>,
        car: &EntrylistCar,
    ) {
        debug!("Received car information packet for car ID {}", car.id);
    }

    fn track_data<M: MessageHandler>(
        &self,
        _client: &BroadcastingClient<M>,
        track_data: &TrackData,
    ) {
        debug!("Received track data packet for {}", track_data.name);
    }

    fn broadcasting_event<M: MessageHandler>(
        &self,
        _client: &BroadcastingClient<M>,
        event: &BroadcastingEvent,
    ) {
        debug!("Received broadcasting event {:?}", event.event_type);
    }
}

pub struct BroadcastingClient<H: MessageHandler> {
    connection_id: u32,
    socket: UdpSocket,
    context: Context,
    handler: H,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Server returned registration error: {0}")]
    RegistrationError(String),
    #[error("Failed decoding packet at location {0:?}")]
    MessageDecodeError(ErrorTree<ByteOffset>),
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
}

impl<H> BroadcastingClient<H>
where
    H: MessageHandler,
{
    pub fn connect<A: ToSocketAddrs, B: ToSocketAddrs>(
        listen: A,
        remote: B,
        handler: H,
        req: RegistrationRequest,
    ) -> Result<Self, ClientError> {
        // Bind the listening socket first
        let socket = UdpSocket::bind(listen)?;
        socket.connect(remote)?;

        // Then transmit the registration request
        let mut buffer = vec![];
        req.encode(&mut buffer)?;
        socket.send(&buffer)?;

        // Set a 1s timeout and wait for the registration reply to come back, if we get something else, panic for now
        socket.set_read_timeout(Some(Duration::from_secs(1)))?;
        let mut incoming = vec![0u8; UDP_MAX];
        let size = socket.recv(&mut incoming)?;

        let packet = &incoming[..size];

        let connection_id: u32 = match InboundMessage::decode(&packet) {
            Ok(InboundMessage::RegistrationResult(res)) => {
                if res.connection_success {
                    info!("Successfully registered with ACC Server");
                    Ok(res.connection_id)
                } else {
                    Err(ClientError::RegistrationError(
                        res.error_message.to_string(),
                    ))
                }
            }
            Ok(_) => {
                // Without the connection ID we can't shut down later on, so we cannot continue
                panic!("Recevied a non-registration reply as first incoming packet!");
            }
            Err(e) => Err(ClientError::MessageDecodeError(e)),
        }?;

        // Put the socket back in blocking mode
        socket.set_read_timeout(None)?;

        Ok(Self {
            connection_id,
            socket,
            context: Context::new(),
            handler,
        })
    }

    pub fn send<M>(&self, message: M) -> Result<(), std::io::Error>
    where
        M: OutboundMessage<Vec<u8>>,
    {
        // 64 bytes accomodates almost every outbound message type
        let mut buffer = Vec::with_capacity(64);
        message.encode(&mut buffer)?;
        self.socket.send(&buffer)?;

        Ok(())
    }

    pub fn ctx(&self) -> &Context {
        &self.context
    }

    pub fn shutdown(self) -> Result<(), std::io::Error> {
        let unregister = UnregisterRequest::new(self.connection_id);
        self.send(unregister)
    }

    pub fn poll(&mut self) -> Result<(), ClientError> {
        let mut buffer = vec![0u8; UDP_MAX];
        let size = self.socket.recv(&mut buffer)?;
        let decoded =
            InboundMessage::decode(&buffer[..size]).map_err(ClientError::MessageDecodeError)?;

        match decoded {
            InboundMessage::RealtimeUpdate(rt) => self.handler.realtime_update(&self, &rt),
            InboundMessage::RealtimeCarUpdate(rt) => {
                self.context.update_car_state(rt.clone());
                self.handler.realtime_car_update(&self, &rt)
            }
            InboundMessage::EntrylistUpdate(list) => {
                self.context.seed_entrylist(&list);
                self.handler.entrylist_update(&self, &list)
            }
            InboundMessage::EntrylistCar(car) => {
                self.context.update_car_entry(car.clone());
                self.handler.entrylist_car(&self, &car)
            }
            InboundMessage::TrackData(track) => {
                self.context.update_track_data(track.clone());
                self.handler.track_data(&self, &track)
            }
            InboundMessage::BroadcastingEvent(event) => {
                self.handler.broadcasting_event(&self, &event)
            }
            InboundMessage::RegistrationResult(_) => (),
        }
        Ok(())
    }
}
