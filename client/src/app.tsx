import {blue, teal} from '@mui/material/colors';
import {
  createTheme,
  ThemeProvider,
  Theme
} from '@mui/material/styles';
import {createStyles, makeStyles} from '@mui/styles';
import * as React from 'react';
import {Route, Routes} from 'react-router';
import {BrowserRouter} from 'react-router-dom';
import {Box} from '@mui/material';
import {GameListPage} from './pages/GameListPage';
import {GamePage} from './pages/GamePage';
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

  return (
    <Box sx={{backgroundColor: 'background.default', color: 'text.primary'}} className={classes.root}>
      {/* This meta tag makes the mobile experience
      much better by preventing text from being tiny. */}
      <meta name='viewport' content='width=device-width, initial-scale=1.0'/>
      <div className={classes.pageContent}>
        <BrowserRouter>
          <Routes>
            <Route
              path='/gameList'
              element={<GameListPage/>}
            />
            <Route
              path='/game'
              element={<GamePage/>}
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