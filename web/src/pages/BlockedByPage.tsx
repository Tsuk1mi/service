import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  CircularProgress,
  IconButton,
  Stack,
  Typography,
} from '@mui/material';
import PhoneIcon from '@mui/icons-material/Phone';
import SmsIcon from '@mui/icons-material/Sms';
import TelegramIcon from '@mui/icons-material/Telegram';
import RefreshIcon from '@mui/icons-material/Refresh';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import { formatDateTime, formatTime } from '../utils/date';
import { formatPlate } from '../utils/plate';
import { openPhone, openSms, openTelegram } from '../utils/contacts';

export function BlockedByPage() {
  const blocksQuery = useQuery({
    queryKey: ['blocked-by'],
    queryFn: () => api.getBlocksForMyPlate(),
  });

  return (
    <Box p={2} maxWidth={700} mx="auto">
      <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
        <Typography variant="h6">Меня заблокировали</Typography>
        <IconButton onClick={() => void blocksQuery.refetch()}>
          <RefreshIcon />
        </IconButton>
      </Stack>

      {blocksQuery.error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {(blocksQuery.error as Error).message}
        </Alert>
      )}

      {blocksQuery.isLoading ? (
        <Box display="flex" justifyContent="center" p={4}>
          <CircularProgress />
        </Box>
      ) : (
        <Stack spacing={1.5}>
          {(blocksQuery.data ?? []).map((block) => {
            const blocker = block.blocker;
            const hasPhone = !!blocker.phone;
            const hasTelegram = !!blocker.telegram;

            return (
              <Card key={block.id} variant="outlined">
                <CardContent>
                  <Typography variant="overline" color="text.secondary">
                    Ваш авто: {formatPlate(block.blocked_plate)}
                  </Typography>
                  <Typography variant="h6" gutterBottom>
                    {blocker.name ?? 'Без имени'}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    Создано: {formatDateTime(block.created_at)}
                  </Typography>
                  {blocker.departure_time && (
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      Время выезда: {formatTime(blocker.departure_time)}
                    </Typography>
                  )}

                  <Stack direction="row" spacing={1} mt={1}>
                    {hasPhone && (
                      <>
                        <Button
                          size="small"
                          variant="outlined"
                          startIcon={<PhoneIcon />}
                          onClick={() => openPhone(blocker.phone!)}
                        >
                          Звонок
                        </Button>
                        <Button
                          size="small"
                          variant="outlined"
                          startIcon={<SmsIcon />}
                          onClick={() =>
                            openSms(blocker.phone!, 'Здравствуйте, мой автомобиль перекрыт.')
                          }
                        >
                          SMS
                        </Button>
                      </>
                    )}
                    {hasTelegram && (
                      <Button
                        size="small"
                        variant="outlined"
                        startIcon={<TelegramIcon />}
                        onClick={() => openTelegram(blocker.telegram!)}
                      >
                        Telegram
                      </Button>
                    )}
                    {!hasPhone && !hasTelegram && (
                      <Typography variant="caption" color="text.secondary">
                        Контакты скрыты
                      </Typography>
                    )}
                  </Stack>
                </CardContent>
              </Card>
            );
          })}
          {(blocksQuery.data ?? []).length === 0 && (
            <Typography color="text.secondary" textAlign="center" py={4}>
              Никто не блокирует ваши автомобили
            </Typography>
          )}
        </Stack>
      )}
    </Box>
  );
}
