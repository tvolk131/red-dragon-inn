import {Button, Card, CardActions, CardContent, Checkbox, Stack} from '@mui/material';
import * as React from 'react';
import {useEffect, useState} from 'react';
import {discardCards, GameView, playCard} from '../../api';

interface HandProps {
  gameView: GameView;
  canDiscardCards: boolean;
}

export const Hand = (props: HandProps) => {
  const [selectedCardIndices, setSelectedCardIndices] = useState<number[]>([]);

  useEffect(() => {
    if (!props.canDiscardCards) {
      setSelectedCardIndices([]);
    }
  }, [props.canDiscardCards]);

  return (
    <div style={{margin: 'auto', width: 'fit-content'}}>
      <Stack direction={'row'}>
        {props.gameView.hand.map((card, index) => {
          return (
            <Card style={{margin: '10px'}}>
              <CardContent>
                {card.cardName}
                {props.canDiscardCards && <Checkbox onChange={(event) => {
                  if (event.target.checked) {
                    setSelectedCardIndices([...selectedCardIndices, index]);
                  } else {
                    setSelectedCardIndices(selectedCardIndices.filter((item) => item !== index));
                  }
                }} checked={selectedCardIndices.includes(index)}/>}
              </CardContent>
              {card.isPlayable && (card.isDirected ? (
                props.gameView?.playerData.map((playerData) => {
                  return (
                    <CardActions>
                      <Button onClick={() => playCard(index, playerData.playerUuid)}>
                        Play (Direct at {props.gameView?.playerDisplayNames[playerData.playerUuid]})
                      </Button>
                    </CardActions>
                  );
                })
              ) : (
                <CardActions>
                  <Button onClick={() => playCard(index)}>
                    Play
                  </Button>
                </CardActions>
              ))}
            </Card>
          );
        })}
      </Stack>
      {props.canDiscardCards && (
        <Button onClick={() => discardCards(selectedCardIndices).then(() => setSelectedCardIndices([]))}>
          Discard {selectedCardIndices.length} cards
        </Button>
      )}
    </div>
  );
};