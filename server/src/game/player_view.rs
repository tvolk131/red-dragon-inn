use super::player_card::PlayerCard;

pub struct GameViewPlayerCard {
    card_name: String,
    is_playable: bool,
}

pub struct GameView {
    hand: Vec<GameViewPlayerCard>,
    alcohol_content: i32,
    fortitude: i32,
    gold: i32,
}
