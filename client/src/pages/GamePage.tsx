import {Button, Card, CardContent, Paper, Typography} from '@mui/material';
import * as React from 'react';
import {useEffect, useState} from 'react';
import {useNavigate} from 'react-router';
import {GameView, orderDrink, pass, selectCharacter, startGame} from '../api';
import {Hand} from './gamePage/Hand';

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
  const [canDiscardCards, setCanDiscardCards] = useState(getCanDiscardCards(props.gameView));

  const navigate = useNavigate();

  const canOrderDrinks = props.gameView
                      && props.gameView.currentTurnPlayerUuid === props.gameView.selfPlayerUuid
                      && props.gameView.currentTurnPhase === 'OrderDrinks';

  useEffect(() => {
    setCanDiscardCards(getCanDiscardCards(props.gameView));
  }, [props.gameView]);

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
      <Button onClick={() => startGame()} disabled={props.gameView.isRunning}>
        Start Game
      </Button>
      {(!props.gameView.isRunning) &&
        <div>
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
        </div>
      }
      <Typography>{props.gameView.isRunning ? 'Game is running' : 'Game is not running'}</Typography>
      {props.gameView.winnerUuid &&
        <Typography>Winner: {props.gameView.playerDisplayNames[props.gameView.winnerUuid]}</Typography>}
      {props.gameView.playerData.map((playerData) => (
        <Card>
          <CardContent>
            <Typography gutterBottom variant='h5' component='div'>
              {props.gameView?.playerDisplayNames[playerData.playerUuid]}
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
      ))}
      {props.gameView.drinkEvent && <Paper>
        <Typography>Drink Event: {props.gameView.drinkEvent.eventName}</Typography>
        {props.gameView.drinkEvent.drinkingContestRemainingPlayerUuids &&
          <div>
            <Typography>Drinking contest remaining contestants</Typography>
            {props.gameView.drinkEvent.drinkingContestRemainingPlayerUuids.map((playerUuid) =>
              <Typography>{props.gameView?.playerDisplayNames[playerUuid]}</Typography>)}
          </div>
        }
      </Paper>}
      {props.gameView.interrupts && <Paper>
        <Typography>Game Interrupts</Typography>
        <Typography>
          Current interrupt turn: {props.gameView.playerDisplayNames[props.gameView.interrupts.currentInterruptTurn]}
        </Typography>
        {props.gameView.interrupts.interrupts.map((interrupt) => (
          <Paper>
            <Typography>Root {interrupt.rootItem.itemType}: {interrupt.rootItem.name}</Typography>
            {interrupt.interruptCardNames.map((interruptCardName) => (
              <Typography>Interrupt card: {interruptCardName}</Typography>
            ))}
          </Paper>
        ))}
      </Paper>}
      <Hand gameView={props.gameView} canDiscardCards={canDiscardCards}/>
      <Button disabled={!props.gameView.canPass} onClick={() => pass()}>Pass</Button>
      {props.gameView.currentTurnPlayerUuid ?
        <Typography>{props.gameView.playerDisplayNames[props.gameView.currentTurnPlayerUuid]}'s turn</Typography> :
        <div>Game not running</div>}
      {canOrderDrinks && (<div>
        {props.gameView.playerData
          .filter((player) => player.playerUuid !== props.gameView?.selfPlayerUuid)
          .map((player) => {
            const playerDisplayName = props.gameView?.playerDisplayNames[player.playerUuid];
            return (
              <Button onClick={() => orderDrink(player.playerUuid)}>
                Order drink for {playerDisplayName}
              </Button>
            );
          })
        }
      </div>)}
    </div>
  );
};
