use soroban_sdk::{contracttype, contracterror, Bytes, Env, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyHasIdentity = 1,
    NoIdentityFound = 2,
    InvalidTier = 3,
    InvalidNonce = 4,
    InvalidSignature = 5,
    InsufficientPayment = 6,
    TransferNotAllowed = 7,
    EmptyUsername = 8,
    NotInitialized = 9,
    NotAdmin = 10,
    TokenNotFound = 11,
    AccessControlError = 12,
    Unauthorized = 13,
    AlreadyInitialized = 14,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GithubData {
    pub username: String,
    pub contributions: u32,
    pub tier: Tier,
    pub minted_at: u64,
    pub updated_at: u64,
    pub proof_data: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tier {
    Novice,
    Pro,
    Architect,
    Legend,
    Singularity,
}

impl Tier {
    pub fn from_contributions(contributions: u32) -> Self {
        match contributions {
            5000.. => Tier::Singularity,
            3000..=4999 => Tier::Legend,
            1000..=2999 => Tier::Architect,
            200..=999 => Tier::Pro,
            _ => Tier::Novice,
        }
    }

    pub fn to_number(&self) -> u8 {
        match self {
            Tier::Novice => 1,
            Tier::Pro => 2,
            Tier::Architect => 3,
            Tier::Legend => 4,
            Tier::Singularity => 5,
        }
    }

    pub fn to_string(&self, env: &Env) -> String {
        match self {
            Tier::Novice => String::from_str(env, "Novice"),
            Tier::Pro => String::from_str(env, "Pro"),
            Tier::Architect => String::from_str(env, "Architect"),
            Tier::Legend => String::from_str(env, "Legend"),
            Tier::Singularity => String::from_str(env, "Singularity"),
        }
    }

    pub fn to_color(&self, env: &Env) -> String {
        match self {
            Tier::Novice => String::from_str(env, "#CD7F32"),
            Tier::Pro => String::from_str(env, "#C0C0C0"),
            Tier::Architect => String::from_str(env, "#FFD700"),
            Tier::Legend => String::from_str(env, "#E5E4E2"),
            Tier::Singularity => String::from_str(env, "#39FF14"),
        }
    }
}

#[contracttype]
#[derive(Clone)]
pub struct Config {
    pub admin: soroban_sdk::Address,
    pub access_control: soroban_sdk::Address,
    pub treasury: soroban_sdk::Address,
    pub mint_fee: i128,
}

pub fn generate_svg(env: &Env, data: &GithubData) -> String {
    let svg = match data.tier {
        Tier::Novice => "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#b0c4de'/><text x='50%' y='100' font-size='24' fill='#181c2f' text-anchor='middle'>Novice</text></svg>",
        Tier::Pro => "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#90ee90'/><text x='50%' y='100' font-size='24' fill='#181c2f' text-anchor='middle'>Pro</text></svg>",
        Tier::Architect => "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#ffd700'/><text x='50%' y='100' font-size='24' fill='#181c2f' text-anchor='middle'>Architect</text></svg>",
        Tier::Legend => "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#ff8c00'/><text x='50%' y='100' font-size='24' fill='#fff' text-anchor='middle'>Legend</text></svg>",
        Tier::Singularity => "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#8a2be2'/><text x='50%' y='100' font-size='24' fill='#fff' text-anchor='middle'>Singularity</text></svg>",
    };
    String::from_str(env, svg)
}