use crate::DecodeError;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum SessionType {
    Practice,
    Qualifying,
    Superpole,
    Race,
    Hotlap,
    Hotstint,
    HotlapSuperpole,
    Replay,
}

impl TryFrom<u8> for SessionType {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SessionType::Practice),
            4 => Ok(SessionType::Qualifying),
            9 => Ok(SessionType::Superpole),
            10 => Ok(SessionType::Race),
            11 => Ok(SessionType::Hotlap),
            12 => Ok(SessionType::Hotstint),
            13 => Ok(SessionType::HotlapSuperpole),
            14 => Ok(SessionType::Replay),
            x => Err(DecodeError::UnknownSessionType(x)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SessionPhase {
    None,
    Starting,
    PreFormation,
    FormationLap,
    PreSession,
    Session,
    SessionOver,
    PostSession,
    ResultUi,
}

impl TryFrom<u8> for SessionPhase {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SessionPhase::None),
            1 => Ok(SessionPhase::Starting),
            2 => Ok(SessionPhase::PreFormation),
            3 => Ok(SessionPhase::FormationLap),
            4 => Ok(SessionPhase::PreSession),
            5 => Ok(SessionPhase::Session),
            6 => Ok(SessionPhase::SessionOver),
            7 => Ok(SessionPhase::PostSession),
            8 => Ok(SessionPhase::ResultUi),
            x => Err(DecodeError::UnknownSessionPhase(x)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CarLocation {
    None,
    Track,
    Pitlane,
    PitEntry,
    PitExit,
}

impl TryFrom<u8> for CarLocation {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CarLocation::None),
            1 => Ok(CarLocation::Track),
            2 => Ok(CarLocation::Pitlane),
            3 => Ok(CarLocation::PitEntry),
            4 => Ok(CarLocation::PitExit),
            x => Err(DecodeError::UnknownCarLocation(x)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Nationality {
    Any,
    Italy,
    Germany,
    France,
    Spain,
    GreatBritain,
    Hungary,
    Belgium,
    Switzerland,
    Austria,
    Russia,
    Thailand,
    Netherlands,
    Poland,
    Argentina,
    Monaco,
    Ireland,
    Brazil,
    SouthAfrica,
    PuertoRico,
    Slovakia,
    Oman,
    Greece,
    SaudiArabia,
    Norway,
    Turkey,
    SouthKorea,
    Lebanon,
    Armenia,
    Mexico,
    Sweden,
    Finland,
    Denmark,
    Croatia,
    Canada,
    China,
    Portugal,
    Singapore,
    Indonesia,
    Usa,
    NewZealand,
    Australia,
    SanMarino,
    Uae,
    Luxembourg,
    Kuwait,
    HongKong,
    Colombia,
    Japan,
    Andorra,
    Azerbaijan,
    Bulgaria,
    Cuba,
    CzechRepublic,
    Estonia,
    Georgia,
    India,
    Israel,
    Jamaica,
    Latvia,
    Lithuania,
    Macau,
    Malaysia,
    Nepal,
    NewCaledonia,
    Nigeria,
    NorthernIreland,
    PapuaNewGuinea,
    Philippines,
    Qatar,
    Romania,
    Scotland,
    Serbia,
    Slovenia,
    Taiwan,
    Ukraine,
    Venezuela,
    Wales,
}

impl TryFrom<u16> for Nationality {
    type Error = DecodeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Nationality::Any),
            1 => Ok(Nationality::Italy),
            2 => Ok(Nationality::Germany),
            3 => Ok(Nationality::France),
            4 => Ok(Nationality::Spain),
            5 => Ok(Nationality::GreatBritain),
            6 => Ok(Nationality::Hungary),
            7 => Ok(Nationality::Belgium),
            8 => Ok(Nationality::Switzerland),
            9 => Ok(Nationality::Austria),
            10 => Ok(Nationality::Russia),
            11 => Ok(Nationality::Thailand),
            12 => Ok(Nationality::Netherlands),
            13 => Ok(Nationality::Poland),
            14 => Ok(Nationality::Argentina),
            15 => Ok(Nationality::Monaco),
            16 => Ok(Nationality::Ireland),
            17 => Ok(Nationality::Brazil),
            18 => Ok(Nationality::SouthAfrica),
            19 => Ok(Nationality::PuertoRico),
            20 => Ok(Nationality::Slovakia),
            21 => Ok(Nationality::Oman),
            22 => Ok(Nationality::Greece),
            23 => Ok(Nationality::SaudiArabia),
            24 => Ok(Nationality::Norway),
            25 => Ok(Nationality::Turkey),
            26 => Ok(Nationality::SouthKorea),
            27 => Ok(Nationality::Lebanon),
            28 => Ok(Nationality::Armenia),
            29 => Ok(Nationality::Mexico),
            30 => Ok(Nationality::Sweden),
            31 => Ok(Nationality::Finland),
            32 => Ok(Nationality::Denmark),
            33 => Ok(Nationality::Croatia),
            34 => Ok(Nationality::Canada),
            35 => Ok(Nationality::China),
            36 => Ok(Nationality::Portugal),
            37 => Ok(Nationality::Singapore),
            38 => Ok(Nationality::Indonesia),
            39 => Ok(Nationality::Usa),
            40 => Ok(Nationality::NewZealand),
            41 => Ok(Nationality::Australia),
            42 => Ok(Nationality::SanMarino),
            43 => Ok(Nationality::Uae),
            44 => Ok(Nationality::Luxembourg),
            45 => Ok(Nationality::Kuwait),
            46 => Ok(Nationality::HongKong),
            47 => Ok(Nationality::Colombia),
            48 => Ok(Nationality::Japan),
            49 => Ok(Nationality::Andorra),
            50 => Ok(Nationality::Azerbaijan),
            51 => Ok(Nationality::Bulgaria),
            52 => Ok(Nationality::Cuba),
            53 => Ok(Nationality::CzechRepublic),
            54 => Ok(Nationality::Estonia),
            55 => Ok(Nationality::Georgia),
            56 => Ok(Nationality::India),
            57 => Ok(Nationality::Israel),
            58 => Ok(Nationality::Jamaica),
            59 => Ok(Nationality::Latvia),
            60 => Ok(Nationality::Lithuania),
            61 => Ok(Nationality::Macau),
            62 => Ok(Nationality::Malaysia),
            63 => Ok(Nationality::Nepal),
            64 => Ok(Nationality::NewCaledonia),
            65 => Ok(Nationality::Nigeria),
            66 => Ok(Nationality::NorthernIreland),
            67 => Ok(Nationality::PapuaNewGuinea),
            68 => Ok(Nationality::Philippines),
            69 => Ok(Nationality::Qatar),
            70 => Ok(Nationality::Romania),
            71 => Ok(Nationality::Scotland),
            72 => Ok(Nationality::Serbia),
            73 => Ok(Nationality::Slovenia),
            74 => Ok(Nationality::Taiwan),
            75 => Ok(Nationality::Ukraine),
            76 => Ok(Nationality::Venezuela),
            77 => Ok(Nationality::Wales),
            x => Err(DecodeError::UnknownNationality(x)),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CarModel {
    Porsche911,
    MercedesAMG,
    Ferrari488,
    AudiR8LMS,
    LamborghiniHuracan,
    McLaren650S,
    NissanGTR2018,
    BMWM6,
    BentleyContinental2018,
    Porsche911Cup,
    NissanGTR2017,
    BentleyContinental2016,
    AstonMartinVantageV12,
    LamborghiniGallardo,
    JaguarG3,
    LexusRCF,
    LamborghiniHuracanEvo,
    HondaNSX,
    LamborghiniSuperTrofeo,
    AudiR8LMSEvo,
    AstonMartinVantageV8,
    HondaNSXEvo,
    McLaren720S,
    Porsche911_2,
    Ferrari488Evo,
    MercedesAMGEvo,

    AlpineA1110,
    AstonMartinVantageGT4,
    AudiR8LMSGT4,
    BMWM4GT4,
    ChevroletCamaroGT4,
    GinettaG55GT4,
    KTMXBowGT4,
    MaseratiMCGT4,
    McLaren570SGT4,
    MercedesAMGGT4,
    Porsche718GT4,
}

impl TryFrom<u8> for CarModel {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CarModel::Porsche911),
            1 => Ok(CarModel::MercedesAMG),
            2 => Ok(CarModel::Ferrari488),
            3 => Ok(CarModel::AudiR8LMS),
            4 => Ok(CarModel::LamborghiniHuracan),
            5 => Ok(CarModel::McLaren650S),
            6 => Ok(CarModel::NissanGTR2018),
            7 => Ok(CarModel::BMWM6),
            8 => Ok(CarModel::BentleyContinental2018),
            9 => Ok(CarModel::Porsche911Cup),
            10 => Ok(CarModel::NissanGTR2017),
            11 => Ok(CarModel::BentleyContinental2016),
            12 => Ok(CarModel::AstonMartinVantageV12),
            13 => Ok(CarModel::LamborghiniGallardo),
            14 => Ok(CarModel::JaguarG3),
            15 => Ok(CarModel::LexusRCF),
            16 => Ok(CarModel::LamborghiniHuracanEvo),
            17 => Ok(CarModel::HondaNSX),
            18 => Ok(CarModel::LamborghiniSuperTrofeo),
            19 => Ok(CarModel::AudiR8LMSEvo),
            20 => Ok(CarModel::AstonMartinVantageV8),
            21 => Ok(CarModel::HondaNSXEvo),
            22 => Ok(CarModel::McLaren720S),
            23 => Ok(CarModel::Porsche911_2),
            24 => Ok(CarModel::Ferrari488Evo),
            25 => Ok(CarModel::MercedesAMGEvo),

            50 => Ok(CarModel::AlpineA1110),
            51 => Ok(CarModel::AstonMartinVantageGT4),
            52 => Ok(CarModel::AudiR8LMSGT4),
            53 => Ok(CarModel::BMWM4GT4),
            55 => Ok(CarModel::ChevroletCamaroGT4),
            56 => Ok(CarModel::GinettaG55GT4),
            57 => Ok(CarModel::KTMXBowGT4),
            58 => Ok(CarModel::MaseratiMCGT4),
            59 => Ok(CarModel::McLaren570SGT4),
            60 => Ok(CarModel::MercedesAMGGT4),
            61 => Ok(CarModel::Porsche718GT4),
            x => Err(DecodeError::UnknownCarModel(x)),
        }
    }
}

impl Display for CarModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str_name = match self {
            CarModel::Porsche911 => "Porsche 911 GT3",
            CarModel::MercedesAMG => "Mercedes AMG GT3",
            CarModel::Ferrari488 => "Ferrari 488 GT3",
            CarModel::AudiR8LMS => "Audi R8 LMS",
            CarModel::LamborghiniHuracan => "Lamborghini Huracan GT3",
            CarModel::McLaren650S => "McLaren 650s GT3",
            CarModel::NissanGTR2018 => "Nissan GT R Nismo GT3 2018",
            CarModel::BMWM6 => "BMW M6 GT3",
            CarModel::BentleyContinental2018 => "Bentley Continental GT3 2018",
            CarModel::Porsche911Cup => "Porsche 991.2 GT3 Cup",
            CarModel::NissanGTR2017 => "Nissan GT-R Nismo GT3 2017",
            CarModel::BentleyContinental2016 => "Bentley Continental GT3 2016",
            CarModel::AstonMartinVantageV12 => "Aston Martin Vantage V12 GT3",
            CarModel::LamborghiniGallardo => "Lamborghini Gallardo R-EX",
            CarModel::JaguarG3 => "Jaguar G3",
            CarModel::LexusRCF => "Lexus RC F GT3",
            CarModel::LamborghiniHuracanEvo => "Lamborghini Huracan Evo (2019)",
            CarModel::HondaNSX => "Honda NSX GT3",
            CarModel::LamborghiniSuperTrofeo => "Lamborghini Huracan SuperTrofeo",
            CarModel::AudiR8LMSEvo => "Audi R8 LMS Evo (2019)",
            CarModel::AstonMartinVantageV8 => "AMR V8 Vantage (2019)",
            CarModel::HondaNSXEvo => "Honda NSX Evo (2019)",
            CarModel::McLaren720S => "McLaren 720S GT3 (2019)",
            CarModel::Porsche911_2 => "Porsche 911 II GT3 R (2019)",
            CarModel::Ferrari488Evo => "Ferrari 488 GT3 Evo 2020",
            CarModel::MercedesAMGEvo => "Mercedes-AMG GT3 Evo 2020",

            CarModel::AlpineA1110 => "Alpine A1110 GT4",
            CarModel::AstonMartinVantageGT4 => "Aston Martin Vantage GT4",
            CarModel::AudiR8LMSGT4 => "Audi R8 LMS GT4",
            CarModel::BMWM4GT4 => "BMW M4 GT4",
            CarModel::ChevroletCamaroGT4 => "Chevrolet Camaro GT4",
            CarModel::GinettaG55GT4 => "Ginetta G55 GT4",
            CarModel::KTMXBowGT4 => "KTM X-Bow GT4",
            CarModel::MaseratiMCGT4 => "Maserati MC GT4",
            CarModel::McLaren570SGT4 => "McLaren 570S GT4",
            CarModel::MercedesAMGGT4 => "Mercedes AMG GT4",
            CarModel::Porsche718GT4 => "Porsche 718 Cayman GT4",
        };
        write!(f, "{}", str_name)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DriverCategory {
    Platinum,
    Gold,
    Silver,
    Bronze,
}

impl TryFrom<u8> for DriverCategory {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(Self::Platinum),
            2 => Ok(Self::Gold),
            1 => Ok(Self::Silver),
            0 => Ok(Self::Bronze),
            x => Err(DecodeError::UnknownDriverCategory(x)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CupCategory {
    Overall,
    ProAm,
    Am,
    Silver,
    National,
}

impl TryFrom<u8> for CupCategory {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Overall),
            1 => Ok(Self::ProAm),
            2 => Ok(Self::Am),
            3 => Ok(Self::Silver),
            4 => Ok(Self::National),
            x => Err(DecodeError::UnknownCupCategory(x)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BroadcastingEventType {
    None,
    GreenFlag,
    SessionOver,
    PenaltyMessage,
    Accident,
    LapCompleted,
    BestSessionLap,
    BestPersonalLap,
}

impl TryFrom<u8> for BroadcastingEventType {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::GreenFlag),
            2 => Ok(Self::SessionOver),
            3 => Ok(Self::PenaltyMessage),
            4 => Ok(Self::Accident),
            5 => Ok(Self::LapCompleted),
            6 => Ok(Self::BestSessionLap),
            7 => Ok(Self::BestPersonalLap),
            x => Err(DecodeError::UnknownBroadcastingEvent(x)),
        }
    }
}
