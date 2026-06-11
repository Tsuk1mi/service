import { useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  CircularProgress,
  Link,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import LockIcon from '@mui/icons-material/Lock';
import TelegramIcon from '@mui/icons-material/Telegram';
import { api } from '../api/client';
import { useAuth } from '../auth/AuthContext';
import { normalizePhone, validatePhone } from '../utils/phone';
import { openTelegramDeeplink } from '../utils/contacts';

export function LoginPage() {
  const { login } = useAuth();
  const [phone, setPhone] = useState('');
  const [code, setCode] = useState('');
  const [codeSent, setCodeSent] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [telegramDeeplink, setTelegramDeeplink] = useState<string | null>(null);
  const [telegramUsername, setTelegramUsername] = useState<string | null>(null);

  const handleStartAuth = async () => {
    setError(null);
    if (!validatePhone(phone)) {
      setError('Введите корректный номер телефона');
      return;
    }
    setLoading(true);
    try {
      const normalized = normalizePhone(phone);
      const response = await api.authStart({ phone: normalized });
      setCodeSent(true);
      setTelegramDeeplink(response.telegram_deeplink ?? null);
      setTelegramUsername(response.telegram_bot_username ?? null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Ошибка отправки кода');
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    setError(null);
    if (!code.trim()) {
      setError('Введите код');
      return;
    }
    setLoading(true);
    try {
      const normalized = normalizePhone(phone);
      const response = await api.authVerify({ phone: normalized, code: code.trim() });
      login(response.token);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Неверный код');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box
      display="flex"
      justifyContent="center"
      alignItems="center"
      minHeight="100vh"
      px={2}
      bgcolor="background.default"
    >
      <Card sx={{ maxWidth: 420, width: '100%' }}>
        <CardContent sx={{ p: 4 }}>
          <Stack spacing={3} alignItems="center">
            <Box
              sx={{
                width: 80,
                height: 80,
                borderRadius: 2,
                bgcolor: 'primary.light',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <LockIcon sx={{ fontSize: 48, color: 'primary.main' }} />
            </Box>
            <Typography variant="h5" fontWeight={600}>
              Rimskiy
            </Typography>
            <Typography variant="body2" color="text.secondary" textAlign="center">
              Вход по SMS-коду или через Telegram-бота
            </Typography>

            {error && <Alert severity="error" sx={{ width: '100%' }}>{error}</Alert>}

            <TextField
              label="Телефон"
              placeholder="+7 (999) 123-45-67"
              value={phone}
              onChange={(e) => setPhone(e.target.value)}
              fullWidth
              disabled={codeSent || loading}
              type="tel"
            />

            {codeSent && (
              <TextField
                label="Код из SMS / Telegram"
                value={code}
                onChange={(e) => setCode(e.target.value)}
                fullWidth
                disabled={loading}
                inputProps={{ maxLength: 6 }}
              />
            )}

            {(telegramDeeplink || telegramUsername) && codeSent && (
              <Alert severity="info" sx={{ width: '100%' }}>
                Код также отправлен в Telegram.
                {telegramDeeplink ? (
                  <Link
                    component="button"
                    variant="body2"
                    onClick={() => openTelegramDeeplink(telegramDeeplink)}
                    sx={{ display: 'flex', alignItems: 'center', gap: 0.5, mt: 1 }}
                  >
                    <TelegramIcon fontSize="small" />
                    Открыть бота
                  </Link>
                ) : telegramUsername ? (
                  <Typography variant="body2" sx={{ mt: 1 }}>
                    @{telegramUsername.replace(/^@/, '')}
                  </Typography>
                ) : null}
              </Alert>
            )}

            {!codeSent ? (
              <Button
                variant="contained"
                fullWidth
                size="large"
                onClick={handleStartAuth}
                disabled={loading}
              >
                {loading ? <CircularProgress size={24} /> : 'Получить код'}
              </Button>
            ) : (
              <Stack direction="row" spacing={1} width="100%">
                <Button variant="outlined" fullWidth onClick={() => setCodeSent(false)} disabled={loading}>
                  Назад
                </Button>
                <Button variant="contained" fullWidth onClick={handleVerify} disabled={loading}>
                  {loading ? <CircularProgress size={24} /> : 'Войти'}
                </Button>
              </Stack>
            )}
          </Stack>
        </CardContent>
      </Card>
    </Box>
  );
}
