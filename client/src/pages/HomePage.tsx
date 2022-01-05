import * as React from 'react';
import {Navigate} from 'react-router';

export const HomePage = () => {
  return (
    <Navigate to="/gameList" replace/>
  );
};