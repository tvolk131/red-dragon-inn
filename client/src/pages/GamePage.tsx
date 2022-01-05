import * as React from 'react';
import {Button, Card, CardContent, Typography} from '@mui/material';
import {GameView, selectCharacter, startGame} from '../api';
import {useNavigate} from 'react-router';

enum Character {
  Fiona,
  Zot,
  Deirdre,
  Gerki
}

const characterToString = (character: Character): string => {
  switch (character) {
    case Character.Fiona: return 'Fiona'
    case Character.Zot: return 'Zot'
    case Character.Deirdre: return 'Deirdre'
    case Character.Gerki: return 'Gerki'
  }
}

interface GamePageProps {
  gameView?: GameView;
}

export const GamePage = (props: GamePageProps) => {
  const navigate = useNavigate();

  if (!props.gameView) {
    return (
      <div>
        <Typography>You are not in a game!</Typography>
        <Button onClick={() => navigate('/gameList')}>Join a game</Button>
      </div>
    );
  }

  return (
    <div>
      <Typography>Game: {props.gameView.gameName}</Typography>
      <Button onClick={() => startGame()}>
        Start Game
      </Button>
      <Button onClick={() => selectCharacter(characterToString(Character.Fiona))}>
        Select Fiona
      </Button>
      <Button onClick={() => selectCharacter(characterToString(Character.Zot))}>
        Select Zot
      </Button>
      <Button onClick={() => selectCharacter(characterToString(Character.Deirdre))}>
        Select Deirdre
      </Button>
      <Button onClick={() => selectCharacter(characterToString(Character.Gerki))}>
        Select Gerki
      </Button>
      {props.gameView.playerData.map((playerData) => {
        const playerDisplayName = props.gameView?.playerDisplayNames[playerData.playerUuid];
        return (
          <Card>
            <CardContent>
              <Typography gutterBottom variant="h5" component="div">
                {playerDisplayName}
              </Typography>
              <Typography>
                Draw Pile Size: {playerData.drawPileSize}
              </Typography>
              <Typography>
                Discard Pile Size: {playerData.discardPileSize}
              </Typography>
              <Typography>
                Drink Deck Size: {playerData.drinkDeckSize}
              </Typography>
              <Typography>
                Alcohol Content: {playerData.alcoholContent}
              </Typography>
              <Typography>
                Fortitude: {playerData.fortitude}
              </Typography>
              <Typography>
                Gold: {playerData.gold}
              </Typography>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
};