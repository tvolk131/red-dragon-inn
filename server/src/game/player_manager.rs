use super::player::Player;
use super::player_card::PlayerCard;
use super::player_view::GameViewPlayerData;
use super::uuid::PlayerUUID;
use super::Character;

#[derive(Clone, Debug)]
pub struct PlayerManager {
    players: Vec<(PlayerUUID, Player)>,
}

impl PlayerManager {
    pub fn new(players_with_characters: Vec<(PlayerUUID, Character)>) -> Self {
        let player_count = players_with_characters.len();

        PlayerManager {
            players: players_with_characters
                .into_iter()
                .map(|(player_uuid, character)| {
                    (
                        player_uuid,
                        Player::create_from_character(
                            character,
                            Self::get_starting_gold_amount_for_player_count(player_count),
                        ),
                    )
                })
                .collect(),
        }
    }

    pub fn clone_uuids_of_all_alive_players(&self) -> Vec<PlayerUUID> {
        self.players
            .iter()
            .filter(|(_, player)| !player.is_out_of_game())
            .map(|(player_uuid, _)| player_uuid)
            .cloned()
            .collect()
    }

    pub fn iter_mut_players(&mut self) -> std::slice::IterMut<(PlayerUUID, Player)> {
        self.players.iter_mut()
    }

    pub fn get_player_by_uuid(&self, player_uuid: &PlayerUUID) -> Option<&Player> {
        match self.players.iter().find(|(uuid, _)| uuid == player_uuid) {
            Some((_, player)) => Some(player),
            None => None,
        }
    }

    pub fn get_game_view_player_data_of_all_players(&self) -> Vec<GameViewPlayerData> {
        self.players
            .iter()
            .map(|(player_uuid, player)| player.to_game_view_player_data(player_uuid.clone()))
            .collect()
    }

    pub fn get_player_by_uuid_mut(&mut self, player_uuid: &PlayerUUID) -> Option<&mut Player> {
        match self
            .players
            .iter_mut()
            .find(|(uuid, _)| uuid == player_uuid)
        {
            Some((_, player)) => Some(player),
            None => None,
        }
    }

    pub fn get_next_alive_player_uuid<'a>(
        &'a self,
        player_uuid: &PlayerUUID,
    ) -> NextPlayerUUIDOption<'a> {
        let current_player_index = match self
            .players
            .iter()
            .position(|(uuid, _)| uuid == player_uuid)
        {
            Some(current_player_index) => current_player_index,
            None => return NextPlayerUUIDOption::PlayerNotFound,
        };
        let mut next_player_index = current_player_index + 1;
        if next_player_index == self.players.len() {
            next_player_index = 0;
        }

        let entry = self.players.get(next_player_index).unwrap();
        let mut next_player_uuid = &entry.0;
        let mut next_player = &entry.1;

        while next_player.is_out_of_game() {
            next_player_index += 1;
            if next_player_index == self.players.len() {
                next_player_index = 0;
            }

            let entry = self.players.get(next_player_index).unwrap();
            next_player_uuid = &entry.0;
            next_player = &entry.1;

            if next_player_index == current_player_index {
                return NextPlayerUUIDOption::OnlyPlayerLeft;
            }
        }

        NextPlayerUUIDOption::Some(next_player_uuid)
    }

    pub fn get_running_state(&self) -> GameRunningState {
        let mut remaining_player_uuids = Vec::new();
        for (player_uuid, player) in self.players.iter() {
            if !player.is_out_of_game() {
                remaining_player_uuids.push(player_uuid);
            }
        }

        if remaining_player_uuids.len() > 1 {
            return GameRunningState::Running;
        }

        if let Some(winning_player_uuid) = remaining_player_uuids.first() {
            GameRunningState::Finished(Some((*winning_player_uuid).clone()))
        } else {
            GameRunningState::Finished(None)
        }
    }

    pub fn get_winner_or(&self) -> Option<PlayerUUID> {
        match self.get_running_state() {
            GameRunningState::Running => None,
            GameRunningState::Finished(winner_or) => winner_or,
        }
    }

    pub fn is_game_running(&self) -> bool {
        matches!(self.get_running_state(), GameRunningState::Running)
    }

    pub fn discard_cards(
        &mut self,
        cards: Vec<(PlayerUUID, PlayerCard)>,
    ) -> Result<(), Vec<(PlayerUUID, PlayerCard)>> {
        let mut unhandled_cards = Vec::new();
        for (card_owner_uuid, card) in cards {
            if let Some(card_owner) = self.get_player_by_uuid_mut(&card_owner_uuid) {
                card_owner.discard_card(card);
            } else {
                unhandled_cards.push((card_owner_uuid, card));
            }
        }

        if unhandled_cards.is_empty() {
            Ok(())
        } else {
            Err(unhandled_cards)
        }
    }

    fn get_starting_gold_amount_for_player_count(player_count: usize) -> i32 {
        if player_count <= 2 {
            8
        } else if player_count >= 7 {
            12
        } else {
            10
        }
    }
}

pub enum NextPlayerUUIDOption<'a> {
    Some(&'a PlayerUUID),
    PlayerNotFound,
    OnlyPlayerLeft,
}

pub enum GameRunningState {
    Running,
    Finished(Option<PlayerUUID>), // Contains the winner of the game, if there is one. Is empty if the remaining players all died at the same time.
}
