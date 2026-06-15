export interface AuthStartRequest {
  phone: string;
}

export interface AuthStartResponse {
  code: string;
  expires_in: number;
  telegram_bot_username?: string | null;
  telegram_deeplink?: string | null;
}

export interface AuthVerifyRequest {
  phone: string;
  code: string;
}

export interface AuthVerifyResponse {
  token: string;
  refresh_token: string;
  user_id: string;
}

export interface RefreshTokenRequest {
  token: string;
}

export interface RefreshTokenResponse {
  token: string;
  refresh_token: string;
  user_id: string;
}

export interface UserResponse {
  id: string;
  name?: string | null;
  phone?: string | null;
  telegram?: string | null;
  plate: string;
  show_contacts: boolean;
  owner_type?: string | null;
  owner_info?: Record<string, unknown> | null;
  departure_time?: string | null;
  created_at: string;
}

export interface UpdateUserRequest {
  name?: string | null;
  phone?: string | null;
  telegram?: string | null;
  plate?: string | null;
  show_contacts?: boolean | null;
  owner_type?: string | null;
  owner_info?: Record<string, unknown> | null;
  departure_time?: string | null;
}

export interface PublicUserInfo {
  id: string;
  name?: string | null;
  plate: string;
  phone?: string | null;
  telegram?: string | null;
  departure_time?: string | null;
}

export interface Block {
  id: string;
  blocker_id: string;
  blocked_plate: string;
  created_at: string;
}

export interface CreateBlockRequest {
  blocked_plate: string;
  notify_owner?: boolean;
  departure_time?: string | null;
  notification_method?: string | null;
}

export interface BlockWithBlockerInfo {
  id: string;
  blocked_plate: string;
  created_at: string;
  blocker: PublicUserInfo;
  blocker_owner_type?: string | null;
  blocker_owner_info?: Record<string, unknown> | null;
}

export interface CheckBlockResponse {
  is_blocked: boolean;
  block?: BlockWithBlockerInfo | null;
}

export interface NotificationResponse {
  id: string;
  type?: string | null;
  title?: string | null;
  message?: string | null;
  data?: Record<string, unknown> | null;
  read: boolean;
  created_at?: string | null;
}

export interface UserPlateResponse {
  id: string;
  user_id: string;
  plate: string;
  is_primary: boolean;
  departure_time?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface CreateUserPlateRequest {
  plate: string;
  is_primary?: boolean;
  departure_time?: string | null;
}

export interface UpdateUserPlateRequest {
  departure_time?: string | null;
}

export interface ServerInfoResponse {
  server_url: string;
  port: number;
  server_version: string;
  min_client_version?: string | null;
  release_client_version?: string | null;
  app_download_url?: string | null;
  web_app_url?: string | null;
  telegram_bot_username?: string | null;
}

export interface RecognizePlateResponse {
  success: boolean;
  plate?: string | null;
  error?: string | null;
}

export interface ApiErrorBody {
  error?: string;
  message?: string;
}
