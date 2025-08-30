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
    NonExistentProposalError = 201,
    ProposalClosedError = 202,
    InvalidProposalSupportError = 203,
    VotePeriodNotFinishedError = 204,
    ProposalNotExecutableError = 205,
    TimelockNotMetError = 206,
    ProposalVotePeriodStartedError = 207,
    InsufficientVotingUnitsError = 208,
    AlreadyVotedError = 209,
    InvalidProposalType = 210,
    ProposalAlreadyOpenError = 211,
    OutsideOfVotePeriodError = 212,
    InvalidProposalActionError = 213,
}
