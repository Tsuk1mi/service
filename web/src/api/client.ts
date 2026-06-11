import {
  clearStoredToken,
  formatAuthHeader,
  getStoredToken,
  setStoredToken,
} from '../auth/storage';
import { checkResponse } from './errors';
import type {
  AuthStartRequest,
  AuthStartResponse,
  AuthVerifyRequest,
  AuthVerifyResponse,
  Block,
  BlockWithBlockerInfo,
  CheckBlockResponse,
  CreateBlockRequest,
  CreateUserPlateRequest,
  NotificationResponse,
  PublicUserInfo,
  RecognizePlateResponse,
  RefreshTokenRequest,
  RefreshTokenResponse,
  ServerInfoResponse,
  UpdateUserRequest,
  UserPlateResponse,
  UserResponse,
} from './types';

const BASE_URL = (import.meta.env.VITE_API_URL as string | undefined) ?? '';

function url(path: string): string {
  return `${BASE_URL}${path}`;
}

async function request(
  path: string,
  init: RequestInit = {},
  token?: string | null,
): Promise<Response> {
  const headers = new Headers(init.headers);

  if (token) {
    headers.set('Authorization', formatAuthHeader(token));
  }

  if (init.body && !(init.body instanceof FormData) && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(url(path), { ...init, headers });

  if (response.status === 401 && token) {
    const refreshed = await tryRefreshToken();
    if (refreshed) {
      headers.set('Authorization', formatAuthHeader(refreshed));
      return fetch(url(path), { ...init, headers });
    }
    clearStoredToken();
    window.location.href = '/login';
  }

  return response;
}

async function tryRefreshToken(): Promise<string | null> {
  const current = getStoredToken();
  if (!current) return null;

  try {
    const body: RefreshTokenRequest = { token: current.replace(/^Bearer\s+/i, '') };
    const response = await fetch(url('/api/auth/refresh'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    if (!response.ok) return null;
    const data = (await response.json()) as RefreshTokenResponse;
    setStoredToken(data.token);
    return data.token;
  } catch {
    return null;
  }
}

function authToken(): string | null {
  return getStoredToken();
}

export const api = {
  getServerInfo: async (): Promise<ServerInfoResponse> => {
    const response = await request('/server-info', {
      headers: { 'Cache-Control': 'no-cache' },
    });
    await checkResponse(response);
    return response.json();
  },

  authStart: async (body: AuthStartRequest): Promise<AuthStartResponse> => {
    const response = await request('/api/auth/start', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    await checkResponse(response);
    return response.json();
  },

  authVerify: async (body: AuthVerifyRequest): Promise<AuthVerifyResponse> => {
    const response = await request('/api/auth/verify', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    await checkResponse(response);
    return response.json();
  },

  refreshToken: async (body: RefreshTokenRequest): Promise<RefreshTokenResponse> => {
    const response = await request('/api/auth/refresh', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    await checkResponse(response);
    return response.json();
  },

  getProfile: async (): Promise<UserResponse> => {
    const response = await request('/api/users/me', {
      headers: { 'Cache-Control': 'no-cache' },
    }, authToken());
    await checkResponse(response);
    return response.json();
  },

  updateProfile: async (body: UpdateUserRequest): Promise<UserResponse> => {
    const response = await request('/api/users/me', {
      method: 'PUT',
      body: JSON.stringify(body),
    }, authToken());
    await checkResponse(response);
    return response.json();
  },

  getUserByPlate: async (plate: string): Promise<PublicUserInfo | null> => {
    const response = await request(
      `/api/users/by-plate?plate=${encodeURIComponent(plate)}`,
      {},
      authToken(),
    );
    if (response.status === 404) return null;
    await checkResponse(response);
    return response.json();
  },

  createBlock: async (body: CreateBlockRequest): Promise<Block> => {
    const response = await request('/api/blocks', {
      method: 'POST',
      body: JSON.stringify(body),
    }, authToken());
    await checkResponse(response);
    return response.json();
  },

  getMyBlocks: async (): Promise<Block[]> => {
    const response = await request('/api/blocks', {
      headers: { 'Cache-Control': 'no-cache' },
    }, authToken());
    await checkResponse(response);
    return response.json();
  },

  getBlocksForMyPlate: async (myPlate?: string): Promise<BlockWithBlockerInfo[]> => {
    const query = myPlate ? `?my_plate=${encodeURIComponent(myPlate)}` : '';
    const response = await request(`/api/blocks/my${query}`, {
      headers: { 'Cache-Control': 'no-cache' },
    }, authToken());
    await checkResponse(response);
    return response.json();
  },

  deleteBlock: async (blockId: string): Promise<void> => {
    const response = await request(`/api/blocks/${blockId}`, {
      method: 'DELETE',
    }, authToken());
    await checkResponse(response);
  },

  warnOwner: async (blockId: string): Promise<void> => {
    const response = await request(`/api/blocks/${blockId}/warn-owner`, {
      method: 'POST',
    }, authToken());
    await checkResponse(response);
  },

  checkBlock: async (plate: string): Promise<CheckBlockResponse> => {
    const response = await request(
      `/api/blocks/check?plate=${encodeURIComponent(plate)}`,
      {},
      authToken(),
    );
    await checkResponse(response);
    return response.json();
  },

  recognizePlateFromImage: async (file: File): Promise<RecognizePlateResponse> => {
    const formData = new FormData();
    formData.append('image', file, file.name || 'plate.jpg');
    const response = await request('/api/ocr/recognize-plate-auth', {
      method: 'POST',
      body: formData,
    }, authToken());
    await checkResponse(response);
    return response.json();
  },

  getNotifications: async (unreadOnly = false): Promise<NotificationResponse[]> => {
    const response = await request(
      `/api/notifications?unread_only=${unreadOnly}`,
      { headers: { 'Cache-Control': 'no-cache' } },
      authToken(),
    );
    await checkResponse(response);
    return response.json();
  },

  markNotificationRead: async (notificationId: string): Promise<void> => {
    const response = await request(`/api/notifications/${notificationId}/read`, {
      method: 'PATCH',
    }, authToken());
    await checkResponse(response);
  },

  markAllNotificationsRead: async (): Promise<void> => {
    const response = await request('/api/notifications/read-all', {
      method: 'PATCH',
    }, authToken());
    await checkResponse(response);
  },

  getUserPlates: async (): Promise<UserPlateResponse[]> => {
    const response = await request('/api/user/plates', {
      headers: { 'Cache-Control': 'no-cache' },
    }, authToken());
    await checkResponse(response);
    return response.json();
  },

  createUserPlate: async (body: CreateUserPlateRequest): Promise<UserPlateResponse> => {
    const response = await request('/api/user/plates', {
      method: 'POST',
      body: JSON.stringify(body),
    }, authToken());
    await checkResponse(response);
    return response.json();
  },

  updateUserPlate: async (plateId: string, departureTime: string | null): Promise<void> => {
    const response = await request(`/api/user/plates/${plateId}`, {
      method: 'PATCH',
      body: JSON.stringify({ departure_time: departureTime }),
    }, authToken());
    await checkResponse(response);
  },

  deleteUserPlate: async (plateId: string): Promise<void> => {
    const response = await request(`/api/user/plates/${plateId}`, {
      method: 'DELETE',
    }, authToken());
    await checkResponse(response);
  },

  setPrimaryPlate: async (plateId: string): Promise<void> => {
    const response = await request(`/api/user/plates/${plateId}/primary`, {
      method: 'POST',
    }, authToken());
    await checkResponse(response);
  },
};
