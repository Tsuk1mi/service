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
  Step,
  StepLabel,
  Stepper,
  TextField,
  Typography,
} from '@mui/material';
import LockIcon from '@mui/icons-material/Lock';
import TelegramIcon from '@mui/icons-material/Telegram';
import { api } from '../api/client';
import { useAuth } from '../auth/AuthContext';
import { setStoredTokens } from '../auth/storage';
import { normalizePhone, validatePhone } from '../utils/phone';
import { openTelegramDeeplink } from '../utils/contacts';

const STEPS = ['Телефон', 'Telegram', 'Код'];

export function LoginPage() {
  const { login } = useAuth();
  const [activeStep, setActiveStep] = useState(0);
  const [phone, setPhone] = useState('');
  const [code, setCode] = useState('');
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
      setTelegramDeeplink(response.telegram_deeplink ?? null);
      setTelegramUsername(response.telegram_bot_username ?? null);
      setActiveStep(1);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Ошибка отправки кода');
    } finally {
      setLoading(false);
    }
  };

  const handleOpenTelegram = () => {
    if (telegramDeeplink) {
      openTelegramDeeplink(telegramDeeplink);
    }
    setActiveStep(2);
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
      setStoredTokens(response.token, response.refresh_token);
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
      <Card sx={{ maxWidth: 440, width: '100%' }}>
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
              Бесплатный вход через Telegram
            </Typography>

            <Stepper activeStep={activeStep} alternativeLabel sx={{ width: '100%' }}>
              {STEPS.map((label) => (
                <Step key={label}>
                  <StepLabel>{label}</StepLabel>
                </Step>
              ))}
            </Stepper>

            {error && <Alert severity="error" sx={{ width: '100%' }}>{error}</Alert>}

            {activeStep === 0 && (
              <>
                <TextField
                  label="Телефон"
                  placeholder="+7 (999) 123-45-67"
                  value={phone}
                  onChange={(e) => setPhone(e.target.value)}
                  fullWidth
                  disabled={loading}
                  type="tel"
                />
                <Button
                  variant="contained"
                  fullWidth
                  size="large"
                  onClick={handleStartAuth}
                  disabled={loading}
                >
                  {loading ? <CircularProgress size={24} /> : 'Получить код'}
                </Button>
              </>
            )}

            {activeStep === 1 && (
              <>
                <Alert severity="info" sx={{ width: '100%' }}>
                  Код отправлен в Telegram. Откройте бота и получите код авторизации.
                  {telegramUsername && (
                    <Typography variant="body2" sx={{ mt: 1 }}>
                      Бот: @{telegramUsername.replace(/^@/, '')}
                    </Typography>
                  )}
                </Alert>
                {telegramDeeplink && (
                  <Button
                    variant="contained"
                    fullWidth
                    startIcon={<TelegramIcon />}
                    onClick={handleOpenTelegram}
                  >
                    Открыть Telegram
                  </Button>
                )}
                <Button variant="outlined" fullWidth onClick={() => setActiveStep(2)}>
                  У меня уже есть код
                </Button>
                <Button variant="text" fullWidth onClick={() => setActiveStep(0)}>
                  Назад
                </Button>
              </>
            )}

            {activeStep === 2 && (
              <>
                <TextField
                  label="Код из Telegram"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  fullWidth
                  disabled={loading}
                  inputProps={{ maxLength: 6 }}
                />
                <Stack direction="row" spacing={1} width="100%">
                  <Button variant="outlined" fullWidth onClick={() => setActiveStep(1)} disabled={loading}>
                    Назад
                  </Button>
                  <Button variant="contained" fullWidth onClick={handleVerify} disabled={loading}>
                    {loading ? <CircularProgress size={24} /> : 'Войти'}
                  </Button>
                </Stack>
              </>
            )}
          </Stack>
        </CardContent>
      </Card>
    </Box>
  );
}
