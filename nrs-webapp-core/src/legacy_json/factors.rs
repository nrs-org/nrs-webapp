#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactorScore {
    ActivatedUnpleasant,
    ActivatedPleasant,
    ModerateUnpleasant,
    ModeratePleasant,
    CalmingUnpleasant,
    CalmingPleasant,
    Language,
    Visual,
    Music,
    Boredom,
    Additional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subscore {
    Emotion,
    Art,
    Boredom,
    Additional,
}

impl FactorScore {
    pub const NUM_TOTAL: usize = 11;

    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(FactorScore::ActivatedUnpleasant),
            1 => Some(FactorScore::ActivatedPleasant),
            2 => Some(FactorScore::ModerateUnpleasant),
            3 => Some(FactorScore::ModeratePleasant),
            4 => Some(FactorScore::CalmingUnpleasant),
            5 => Some(FactorScore::CalmingPleasant),
            6 => Some(FactorScore::Language),
            7 => Some(FactorScore::Visual),
            8 => Some(FactorScore::Music),
            9 => Some(FactorScore::Boredom),
            10 => Some(FactorScore::Additional),
            _ => None,
        }
    }

    pub fn from_short_name(name: &str) -> Option<Self> {
        match name {
            "AU" => Some(FactorScore::ActivatedUnpleasant),
            "AP" => Some(FactorScore::ActivatedPleasant),
            "MU" => Some(FactorScore::ModerateUnpleasant),
            "MP" => Some(FactorScore::ModeratePleasant),
            "CU" => Some(FactorScore::CalmingUnpleasant),
            "CP" => Some(FactorScore::CalmingPleasant),
            "AL" => Some(FactorScore::Language),
            "AV" => Some(FactorScore::Visual),
            "AM" => Some(FactorScore::Music),
            "B" => Some(FactorScore::Boredom),
            "A" => Some(FactorScore::Additional),
            _ => None,
        }
    }

    pub fn to_subscore(&self) -> Subscore {
        match self {
            FactorScore::ActivatedUnpleasant
            | FactorScore::ActivatedPleasant
            | FactorScore::ModerateUnpleasant
            | FactorScore::ModeratePleasant
            | FactorScore::CalmingUnpleasant
            | FactorScore::CalmingPleasant => Subscore::Emotion,
            FactorScore::Language | FactorScore::Visual | FactorScore::Music => Subscore::Art,
            FactorScore::Boredom => Subscore::Boredom,
            FactorScore::Additional => Subscore::Additional,
        }
    }

    pub fn all() -> [Self; Self::NUM_TOTAL] {
        [
            FactorScore::ActivatedUnpleasant,
            FactorScore::ActivatedPleasant,
            FactorScore::ModerateUnpleasant,
            FactorScore::ModeratePleasant,
            FactorScore::CalmingUnpleasant,
            FactorScore::CalmingPleasant,
            FactorScore::Language,
            FactorScore::Visual,
            FactorScore::Music,
            FactorScore::Boredom,
            FactorScore::Additional,
        ]
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            FactorScore::ActivatedUnpleasant => "AU",
            FactorScore::ActivatedPleasant => "AP",
            FactorScore::ModerateUnpleasant => "MU",
            FactorScore::ModeratePleasant => "MP",
            FactorScore::CalmingUnpleasant => "CU",
            FactorScore::CalmingPleasant => "CP",
            FactorScore::Language => "AL",
            FactorScore::Visual => "AV",
            FactorScore::Music => "AM",
            FactorScore::Boredom => "B",
            FactorScore::Additional => "A",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}
