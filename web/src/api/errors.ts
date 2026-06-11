import type { ApiErrorBody } from './types';

export async function parseApiError(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as ApiErrorBody;
    return body.error || body.message || `Ошибка ${response.status}`;
  } catch {
    return `Ошибка ${response.status}: ${response.statusText}`;
  }
}

export async function checkResponse(response: Response): Promise<Response> {
  if (!response.ok) {
    throw new Error(await parseApiError(response));
  }
  return response;
}
