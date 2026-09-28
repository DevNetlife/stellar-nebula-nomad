use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub user: Address,
    pub message: Symbol,
    pub timestamp: u64,
}

pub fn emit_notification(env: &Env, player: Address, message: Symbol) {
    let _notification = Notification {
        user: player.clone(),
        message: message.clone(),
        timestamp: env.ledger().timestamp(),
    };

    // Emit event as requested
    env.events().publish(
        (symbol_short!("notify"), player),
        message,
    );
}
