import axios from 'axios';

interface GameViewPlayerCard {
  cardName: string;
  isPlayable: boolean;
}

interface GameViewPlayerData {
  playerUuid: string;
  drawPileSize: number;
  discardPileSize: number;
  drinkDeckSize: number;
  alcoholContent: number;
  fortitude: number;
  gold: number;
}

interface GameView {
  gameName: string;
  selfPlayerId: string;
  hand: GameViewPlayerCard[];
  playerData: GameViewPlayerData[];
  playerDisplayNames: {[key: string]: string}
}

export const signin = async (displayName: string): Promise<void> => {
  await axios.get('/api/signin', {params: {display_name: displayName}});
}

export const signout = async (): Promise<void> => {
  await axios.get('/api/signout');
}

export const createGame = async (gameName: string): Promise<GameView> => {
  return (await axios.get(`/api/createGame/${gameName}`)).data as GameView;
}

export const joinGame = async (gameId: string): Promise<GameView> => {
  return (await axios.get(`/api/joinGame/${gameId}`)).data as GameView;
}

export const leaveGame = async (): Promise<void> => {
  return await axios.get('/api/leaveGame');
}

export const startGame = async (gameId: string): Promise<GameView> => {
  return (await axios.get(`/api/startGame/${gameId}`)).data as GameView;
}

export const selectCharacter = async (character: string): Promise<GameView> => {
  return (await axios.get(`/api/selectCharacter/${character}`)).data as GameView;
}

export const playCard = async (cardIndex: number, otherPlayerUuid?: string): Promise<GameView> => {
  return (await axios.get('/api/playCard', {params: {card_index: cardIndex, other_player_uuid: otherPlayerUuid}})).data as GameView;
}

export const discardCards = async (cardIndices: number[]): Promise<GameView> => {
  return (await axios.get('/api/discardCards', {params: {card_indices_string: cardIndices.join(',')}})).data as GameView;
}

export const orderDrink = async (otherPlayerUuid: string): Promise<GameView> => {
  return (await axios.get(`/api/orderDrink/${otherPlayerUuid}`)).data as GameView;
}

export const pass = async (): Promise<GameView> => {
  return (await axios.get('/api/pass')).data as GameView;
}

export const getGameView = async (): Promise<GameView> => {
  return (await axios.get('/api/getGameView')).data as GameView;
}