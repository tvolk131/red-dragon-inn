import {Button, Card, CardActions, CardContent, CircularProgress, Paper, TextField, Typography} from '@mui/material';
import * as React from 'react';
import {useEffect, useState} from 'react';
import {useNavigate} from 'react-router';
import {createGame, GameView, joinGame, ListedGameView, listGames, signin, signout} from '../api';

interface GameListPageProps {
  displayName: string | undefined;
  gameView?: GameView;
  setDisplayName(displayName: string | undefined): void;
}

export const GameListPage = (props: GameListPageProps) => {
  const navigate = useNavigate();

  const [games, setGames] = useState<ListedGameView[] | undefined>([]);
  const [loadingGames, setLoadingGames] = useState(true);

  useEffect(() => {
    listGames()
      .then((games) => setGames(games))
      .finally(() => setLoadingGames(false));
  }, []);

  return (
    <div>
      <Typography
        variant={'h3'}
      >
        Game List Page
      </Typography>
      {props.displayName === undefined ? <LoginBox {...props}/> : <ProfileBox {...props}/>}
      {props.displayName !== undefined && props.gameView === undefined && <GameCreatorBox/>}
      {props.gameView && <Button onClick={() => navigate('/game')}>View Game</Button>}
      {loadingGames ? <CircularProgress/> : games ? games.map((game) => (
        <Card>
          <CardContent>
            <Typography gutterBottom variant='h5' component='div'>
              {game.gameName}
            </Typography>
          </CardContent>
          <CardActions>
            <Button onClick={() => joinGame(game.gameUuid)}>Join</Button>
          </CardActions>
        </Card>
      )) : <div>Failed to load games. Try refreshing the page.</div>}
    </div>
  );
};

interface LoginBoxProps {
  displayName: string | undefined;
  setDisplayName(displayName: string | undefined): void;
}

const LoginBox = (props: LoginBoxProps) => {
  const [displayName, setDisplayName] = useState('');

  return (
    <div>
      <Paper>You are not logged in. Pick a display name and create a temporary user.</Paper>
      <TextField label={'Display Name'} value={displayName} onChange={(e) => setDisplayName(e.target.value)}/>
      <Button disabled={displayName.length === 0} onClick={() => {
        signin(displayName).then(() => {
          props.setDisplayName(displayName);
          setDisplayName('');
        });
      }}>Login</Button>
    </div>
  );
};

interface ProfileBoxProps {
  displayName: string | undefined;
  setDisplayName(displayName: string | undefined): void;
}

const ProfileBox = (props: ProfileBoxProps) => {
  return (
    <div>
      <Paper>Display Name: {props.displayName}</Paper>
      <Button onClick={() => {
        signout().then(() => props.setDisplayName(undefined));
      }}>Logout</Button>
    </div>
  );
};

const GameCreatorBox = () => {
  const [gameName, setGameName] = useState('');

  return (
    <div>
      <Paper>Create a game!</Paper>
      <TextField label={'Game Name'} value={gameName} onChange={(e) => setGameName(e.target.value)}/>
      <Button disabled={gameName.length === 0} onClick={() => {
        createGame(gameName).then(() => {
          setGameName('');
        });
      }}>Login</Button>
    </div>
  );
};
