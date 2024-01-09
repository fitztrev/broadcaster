export interface LichessBroadcast {
  tour: Tournament;
  round: Round;
  study: Study;
}

interface Tournament {
  id: string;
  name: string;
  slug: string;
  createdAt: number;
  description: string;
  official?: boolean;
}

interface Round {
  id: string;
  name: string;
  slug: string;
  url: string;
  createdAt: number;
  delay?: number;
  finished?: boolean;
  ongoing?: boolean;
  startsAt?: number;
}

interface Study {
  writeable: boolean;
}

export interface AccessTokenResponse {
  token_type: string;
  access_token: string;
  expires_in: number;
}

export interface LichessUser {
  username: string;
}

export interface RoundResponse {
  round: Round;
  tour: Tournament;
  study: Study;
  games: Game[];
}

interface Game {
  id: string;
  name: string;
  ongoing?: boolean;
  res?: string;
  url: string;
}

interface PgnPushResponse {
  moves: number;
}

export interface PgnPushResult {
  response: PgnPushResponse;
  file: string;
}
