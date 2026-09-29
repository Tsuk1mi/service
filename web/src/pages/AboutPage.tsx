import {
  Alert,
  Box,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Link,
  Stack,
  Typography,
} from '@mui/material';
import TelegramIcon from '@mui/icons-material/Telegram';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import { openTelegram } from '../utils/contacts';
import { authChannelHint, parseIntegrations } from '../utils/integrations';

export function AboutPage() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['server-info'],
    queryFn: () => api.getServerInfo(),
  });

  const integrations = parseIntegrations(data);

  if (isLoading) {
    return (
      <Box display="flex" justifyContent="center" p={4}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box p={2} maxWidth={700} mx="auto">
      <Typography variant="h6" gutterBottom>
        О приложении
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {(error as Error).message}
        </Alert>
      )}

      <Card>
        <CardContent>
          <Stack spacing={2}>
            <Box>
              <Typography variant="caption" color="text.secondary">
                Название
              </Typography>
              <Typography variant="body1">Rimskiy Service</Typography>
            </Box>
            <Box>
              <Typography variant="caption" color="text.secondary">
                Версия сервера
              </Typography>
              <Typography variant="body1">{data?.server_version ?? '—'}</Typography>
            </Box>
            <Box>
              <Typography variant="caption" color="text.secondary">
                URL сервера
              </Typography>
              <Typography variant="body1">{data?.server_url ?? '—'}</Typography>
            </Box>
            {data?.web_app_url && (
              <Box>
                <Typography variant="caption" color="text.secondary">
                  Веб-приложение
                </Typography>
                <Typography variant="body1">
                  <Link href={data.web_app_url} target="_blank" rel="noopener noreferrer">
                    {data.web_app_url}
                  </Link>
                </Typography>
              </Box>
            )}
            {data?.telegram_bot_username && (
              <Box>
                <Typography variant="caption" color="text.secondary">
                  Telegram-бот
                </Typography>
                <Link
                  component="button"
                  variant="body1"
                  onClick={() => openTelegram(data.telegram_bot_username!)}
                  sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}
                >
                  <TelegramIcon fontSize="small" />
                  @{data.telegram_bot_username.replace(/^@/, '')}
                </Link>
              </Box>
            )}
            <Box>
              <Typography variant="caption" color="text.secondary" display="block" mb={1}>
                Интеграции
              </Typography>
              <Stack direction="row" flexWrap="wrap" gap={1}>
                <Chip
                  size="small"
                  label={integrations.sms ? 'SMS: включён' : 'SMS: выкл (Telegram-only)'}
                  color={integrations.sms ? 'success' : 'default'}
                  variant="outlined"
                />
                <Chip
                  size="small"
                  label={integrations.ocr ? 'OCR: доступен' : 'OCR: выкл — ручной ввод'}
                  color={integrations.ocr ? 'success' : 'default'}
                  variant="outlined"
                />
                <Chip
                  size="small"
                  label={integrations.telegram ? 'Telegram: ок' : 'Telegram: не настроен'}
                  color={integrations.telegram ? 'success' : 'default'}
                  variant="outlined"
                />
                <Chip
                  size="small"
                  label={integrations.fcm ? 'FCM: ок' : 'FCM: выкл'}
                  color={integrations.fcm ? 'success' : 'default'}
                  variant="outlined"
                />
              </Stack>
            </Box>
            <Typography variant="body2" color="text.secondary">
              Сервис для управления ситуацией «мой автомобиль перекрыли на парковке».{' '}
              {authChannelHint(integrations)}
            </Typography>
          </Stack>
        </CardContent>
      </Card>
    </Box>
  );
}
