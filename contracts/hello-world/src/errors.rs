use soroban_sdk::contracterror;

/// The error codes for the contract.
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BettingError {
    // Default errors to align with built-in contract
    InternalError = 1,
    AlreadyInitializedError = 3,
    InvalidInputError = 5,

    UnauthorizedError = 4,

    NegativeAmountError = 8,
    AllowanceError = 9,
    BalanceError = 10,
    OverflowError = 12,

    GameDoesNotExist = 200,
    SettingBetDoesNotExist = 201,
    PrivateBet_NotAllowToBet = 202,
    PrivateBet_NotEnoughToBet = 203,
    Game_HasFinished = 204,
    Summiters_notAllowToBet = 205,
    PublicSettingNotFound = 206,
    GameHasAlreadyStarted = 207,
    GameIsActive = 208,
    GameHasNotStarted = 209,
    GameHasAlreadySet = 210,
    GameHasNotFinished = 211,
    NotAllowToSummitResult = 212,
    GameAssesmentHasFinished = 213,
    GameResultNotFound = 214,
    UserCannotVote = 215,
    GameHasNotBeenPaused = 216,
    NoBetHasBeenFound = 217,
    NothingToClaim = 218,
    UserAlreadyClaimed = 219,
    GameHasBeenPaused = 220,
    GameHasAlreadyBeenExecuted = 221,
    NotAdmin = 222,
    NotEnoughStake = 223,
    UnknownSigner = 224,
    GameResultAlreadySet = 225,
    BetNotFound = 226,
}
