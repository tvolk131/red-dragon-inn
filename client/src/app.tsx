import {Box} from '@mui/material';
import {blue, teal} from '@mui/material/colors';
import {
  createTheme,
  Theme,
  ThemeProvider
} from '@mui/material/styles';
import {createStyles, makeStyles} from '@mui/styles';
import * as React from 'react';
import {useEffect, useState} from 'react';
import {Route, Routes} from 'react-router';
import {BrowserRouter} from 'react-router-dom';
import {GameView, getGameView, me} from './api';
import {GameListPage} from './pages/GameListPage';
import {GamePage} from './pages/GamePage';
import {HomePage} from './pages/HomePage';
import {NotFoundPage} from './pages/NotFoundPage';

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    root: {
      display: 'flex',
      flexDirection: 'column',
      minHeight: '100vh'
    },
    pageContent: {
      flex: 1
    }
  })
);

const SubApp = () => {
  const classes = useStyles();

  const [displayName, setDisplayName] = useState<string | undefined>(undefined);
  const [loadingDisplayName, setLoadingDisplayName] = useState(true);

  const [gameView, setGameView] = useState<GameView | undefined>(undefined);
  const [loadingGameView, setLoadingGameView] = useState(true);

  useEffect(() => {
    me()
      .then((displayName) => setDisplayName(displayName))
      .catch(() => setDisplayName(undefined))
      .finally(() => setLoadingDisplayName(false));

    getGameView()
      .then((gameView) => setGameView(gameView))
      .catch(() => setGameView(undefined))
      .finally(() => setLoadingGameView(false));

    // TODO - Use websockets instead of intermittent polling.
    setInterval(() => {
      getGameView()
        .then((gameView) => setGameView(gameView));
    }, 500);
  }, []);

  if (loadingDisplayName || loadingGameView) {
    return <div>{loadingDisplayName ? 'Loading...' : displayName}</div>;
  }

  return (
    <Box sx={{backgroundColor: 'background.default', color: 'text.primary'}} className={classes.root}>
      {/* This meta tag makes the mobile experience
      much better by preventing text from being tiny. */}
      <meta name='viewport' content='width=device-width, initial-scale=1.0'/>
      <div className={classes.pageContent}>
        <BrowserRouter>
          <Routes>
            <Route
              path='/'
              element={<HomePage/>}
            />
            <Route
              path='/gameList'
              element={<GameListPage displayName={displayName} setDisplayName={setDisplayName} gameView={gameView}/>}
            />
            <Route
              path='/game'
              element={<GamePage gameView={gameView}/>}
            />
            <Route
              path='*'
              element={<NotFoundPage/>}
            />
          </Routes>
        </BrowserRouter>
      </div>
    </Box>
  );
};

const ThemedSubApp = () => {
  const isDarkMode = true; // TODO - Add a way for users to be able to set this.

  const theme = createTheme({
    palette: {
      primary: blue,
      secondary: teal,
      mode: isDarkMode ? 'dark' : 'light'
    }
  });

  return (
    <ThemeProvider theme={theme}>
      <SubApp/>
    </ThemeProvider>
  );
};

export const App = () => (
  <ThemedSubApp/>
);
