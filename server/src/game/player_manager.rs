use super::player::Player;
use super::uuid::PlayerUUID;
use super::Character;
use super::player_view::GameViewPlayerData;

#[derive(Clone)]
pub struct PlayerManager {
    players: Vec<(PlayerUUID, Player)>
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
            .collect()
        }
    }

    pub fn clone_uuids_of_all_alive_players(&self) -> Vec<PlayerUUID> {
        self.players.iter().filter(|(player_uuid, player)| !player.is_out_of_game()).map(|(player_uuid, _)| player_uuid).cloned().collect()
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

    pub fn get_next_alive_player_uuid<'a>(&'a self, player_uuid: &PlayerUUID) -> NextPlayerUUIDOption<'a> {
        let current_player_index = match self
            .players
            .iter()
            .position(|(uuid, _)| uuid == player_uuid) {
                Some(current_player_index) => current_player_index,
                None => return NextPlayerUUIDOption::PlayerNotFound
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
    OnlyPlayerLeft
}