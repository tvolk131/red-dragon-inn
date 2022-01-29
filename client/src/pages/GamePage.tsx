import {Button, Card, CardActions, CardContent, Checkbox, Typography} from '@mui/material';
import * as React from 'react';
import {useEffect, useState} from 'react';
import {useNavigate} from 'react-router';
import {discardCards, GameView, orderDrink, pass, playCard, selectCharacter, startGame} from '../api';

enum Character {
  Fiona,
  Zot,
  Deirdre,
  Gerki
}

const characterToString = (character: Character): string => {
  switch (character) {
    case Character.Fiona: return 'Fiona';
    case Character.Zot: return 'Zot';
    case Character.Deirdre: return 'Deirdre';
    case Character.Gerki: return 'Gerki';
    default: return 'Unknown Character';
  }
};

const getCanDiscardCards = (gameView?: GameView): boolean => {
  return !!gameView
        && gameView.currentTurnPlayerUuid === gameView.selfPlayerUuid
        && gameView.currentTurnPhase === 'DiscardAndDraw';
};

interface GamePageProps {
  gameView?: GameView;
}

export const GamePage = (props: GamePageProps) => {
  const [selectedCardIndices, setSelectedCardIndices] = useState<number[]>([]);
  const [canDiscardCards, setCanDiscardCards] = useState(getCanDiscardCards(props.gameView));

  const navigate = useNavigate();

  const canOrderDrinks = props.gameView
                      && props.gameView.currentTurnPlayerUuid === props.gameView.selfPlayerUuid
                      && props.gameView.currentTurnPhase === 'OrderDrinks';

  useEffect(() => {
    setCanDiscardCards(getCanDiscardCards(props.gameView));
  }, [props.gameView]);

  useEffect(() => {
    if (!canDiscardCards) {
      setSelectedCardIndices([]);
    }
  }, [canDiscardCards]);

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
              <Typography gutterBottom variant='h5' component='div'>
                {playerDisplayName}
              </Typography>
              <Typography>
                Draw Pile Size: {playerData.drawPileSize}
              </Typography>
              <Typography>
                Discard Pile Size: {playerData.discardPileSize}
              </Typography>
              <Typography>
                Drink Me Pile Size: {playerData.drinkMePileSize}
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
      {props.gameView.hand.map((card, index) => {
        return (
          <Card>
            <CardContent>
              {card.cardName}
              {canDiscardCards && <Checkbox onChange={(event) => {
                if (event.target.checked) {
                  selectedCardIndices.push(index);
                } else {
                  selectedCardIndices.filter((item) => item !== index);
                }
              }} checked={selectedCardIndices.includes(index)}/>}
            </CardContent>
            {card.isPlayable && (
              <CardActions>
                <Button onClick={() => playCard(index)}>
                  Play
                </Button>
              </CardActions>
            )}
          </Card>
        );
      })}
      <Button disabled={!props.gameView.canPass} onClick={() => pass()}>Pass</Button>
      {props.gameView.currentTurnPlayerUuid ?
        <div>{props.gameView.playerDisplayNames[props.gameView.currentTurnPlayerUuid]}'s turn</div> :
        <div>Game not running</div>}
      {canDiscardCards && (
        <Button onClick={() => discardCards(selectedCardIndices).then(() => setSelectedCardIndices([]))}>
          Discard {selectedCardIndices.length} cards
        </Button>
      )}
      {canOrderDrinks && (<div>
        {props.gameView.playerData.map((player) => {
          const playerDisplayName = props.gameView?.playerDisplayNames[player.playerUuid];
          return (
            <Button onClick={() => orderDrink(player.playerUuid)}>
              Order drink for {playerDisplayName}
            </Button>
          );
        })}
      </div>)}
    </div>
  );
};
