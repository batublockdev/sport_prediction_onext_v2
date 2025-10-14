# Betting Contract

This is a Soroban smart contract written in Rust for a decentralized betting platform. It allows users to place bets on games, manage private betting settings, submit game results, assess results, and claim winnings or refunds based on predefined rules. The contract uses a token-based system for bets and stakes, incorporating honesty points and a leaderboard to incentivize fair participation.

## Features

- **Game Setup**: Admins can set up games with start and end times, verified by a cryptographic signature.
- **Private Betting**: Users can create private betting settings with invited participants and minimum bet amounts.
- **Result Submission**: Designated summiters and checkers submit and verify game results, with mechanisms to handle disputes.
- **Result Assessment**: Users and checkers can approve or reject submitted results, influencing the distribution of winnings.
- **Token Management**: Supports USD and trust tokens for betting and staking, with secure token transfers.
- **Honesty Points**: Users earn or lose points based on their betting and assessment behavior, impacting their leaderboard ranking.
- **Leaderboard**: Tracks user scores based on stake amounts and historical performance to select summiters and checkers.
- **Refunds and Claims**: Users can claim refunds if games are not activated or results are not submitted in time, and winners can claim their share of the pool.
- **Supreme Court**: A trusted multi-signature address resolves disputes and sets final results when complaints are raised.

## Key Functions

- `__constructor`: Initializes the contract with admin details, token addresses, and a supreme court address.
- `request_result_summiter`: Allows users to stake tokens to become result summiters.
- `bet`: Enables users to place bets on games, supporting private betting settings.
- `claim_refund`: Processes refunds for unactivated bets or games without timely results.
- `set_game`: Admin function to set up a new game with a verified signature.
- `set_private_bet`: Allows users to create private betting settings for a game.
- `add_user_privateBet`: Adds users to private betting settings.
- `summitResult`: Submits game results, with time-based restrictions and fines for delays.
- `assessResult`: Allows users and checkers to approve or reject game results.
- `claim`: Handles claims for summiters, protocol, or users based on game outcomes.
- `setResult_supremCourt`: Resolves disputes by setting final results via the supreme court address.
- `execute_distribution`: Distributes winnings and fines based on game results and user assessments.
- `set_stakeAmount`: Admin function to set the minimum stake amount for summiters.

## Internal Functions

- `make_distribution`: Distributes pools (winners, losers, protocol) based on game results and complaints.
- `what_kind_user`: Determines user type (e.g., winner, loser, honest, dishonest) for claim processing.
- `select_summiter`: Selects summiters and checkers based on leaderboard rankings.
- `moveToken`: Transfers tokens between addresses securely.
- `adduser_board`: Updates the leaderboard with user scores based on stakes and history.

## Error Handling

The contract uses a `BettingError` enum to handle various error cases, such as:
- Invalid inputs
- Games not existing or already started
- Unauthorized actions (e.g., non-admin attempts)
- Insufficient stakes or bets
- Duplicate actions (e.g., claiming twice)

## Events

The contract emits events via `BettingEvents` for key actions, including:
- Game setup
- Private setting creation
- User addition to private bets
- Result submission
- Result assessment
- Distribution execution
- Honesty points updates
- Summiter selection
- Stake amount changes

## Dependencies

- **Soroban SDK**: For contract development and token interactions.
- **Custom Modules**: 
  - `bettingTrait`: Defines the contract interface.
  - `errors`: Custom error types.
  - `events`: Event emission logic.
  - `storage`: Persistent storage management.
  - `types`: Data structures for bets, games, and assessments.
  - `Constants`: Predefined constants for percentages, points, and time intervals.

## Setup and Deployment

1. **Environment**: Ensure you have the Rust toolchain and Soroban SDK installed.
2. **Compile**: Use `cargo build --target wasm32-unknown-unknown --release` to compile the contract to WebAssembly.
3. **Deploy**: Deploy the contract to a Soroban-compatible network, providing the admin address, public key, USD token address, trust token address, and supreme court address.
4. **Initialize**: Call `__constructor` with the required parameters to set up the contract.

## Usage

- **Admins**: Set games, minimum stake amounts, and resolve disputes via the supreme court.
- **Users**: Stake to become summiters, place bets, assess results, and claim winnings or refunds.
- **Summiters/Checkers**: Submit and verify game results within time constraints to avoid fines.

## Security Considerations

- **Authentication**: Uses `require_auth` to ensure only authorized users perform actions.
- **Signature Verification**: Games are set with admin-signed data to prevent tampering.
- **Time Constraints**: Enforces deadlines for result submission and assessment to ensure timely execution.
- **Fines and Incentives**: Penalizes dishonest or inactive summiters/checkers and rewards honest participation.
- **Token Safety**: Ensures secure token transfers using the Soroban token client.

## License

This contract is provided as-is, with no warranty. Ensure thorough testing and auditing before deploying in a production environment.

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.
