import {
  Box,
  Card,
  CardActionArea,
  CardContent,
  Divider,
  Stack,
  Typography,
} from '@mui/material';
import DirectionsCarIcon from '@mui/icons-material/DirectionsCar';
import PersonIcon from '@mui/icons-material/Person';
import ListIcon from '@mui/icons-material/List';
import WarningIcon from '@mui/icons-material/Warning';
import NotificationsIcon from '@mui/icons-material/Notifications';
import InfoIcon from '@mui/icons-material/Info';
import ArrowForwardIcon from '@mui/icons-material/ArrowForward';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';

const QUICK_ACTIONS = [
  { path: '/profile', title: 'Профиль', subtitle: 'Данные и авто', icon: <PersonIcon /> },
  { path: '/blocks', title: 'Мои блокировки', subtitle: 'Кого перекрыли', icon: <ListIcon /> },
  { path: '/blocked-by', title: 'Меня перекрыл', subtitle: 'Кто блокирует', icon: <WarningIcon /> },
  { path: '/notifications', title: 'Уведомления', subtitle: 'События и алерты', icon: <NotificationsIcon /> },
];

export function HomePage() {
  const navigate = useNavigate();
  const { data: serverInfo } = useQuery({
    queryKey: ['server-info'],
    queryFn: () => api.getServerInfo(),
  });

  return (
    <Box p={2} maxWidth={900} mx="auto">
      <Card sx={{ mb: 2, bgcolor: 'primary.light' }}>
        <CardContent sx={{ p: 3 }}>
          <Stack direction="row" spacing={2} alignItems="center">
            <DirectionsCarIcon sx={{ fontSize: 32, color: 'primary.main' }} />
            <Box flex={1}>
              <Typography variant="h6" color="primary.dark">
                Добро пожаловать!
              </Typography>
              <Typography variant="body2" color="primary.dark" sx={{ opacity: 0.8 }}>
                Управляйте блокировками
              </Typography>
            </Box>
          </Stack>
          <Divider sx={{ my: 2 }} />
          <Stack direction="row" justifyContent="space-between">
            <Box>
              <Typography variant="caption" color="text.secondary">
                Версия сервера
              </Typography>
              <Typography variant="body2">
                {serverInfo?.server_version ?? '—'}
              </Typography>
            </Box>
            {serverInfo?.telegram_bot_username && (
              <Box textAlign="right">
                <Typography variant="caption" color="text.secondary">
                  Telegram-бот
                </Typography>
                <Typography variant="body2">@{serverInfo.telegram_bot_username}</Typography>
              </Box>
            )}
          </Stack>
        </CardContent>
      </Card>

      <Typography variant="h6" gutterBottom>
        Быстрые действия
      </Typography>
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: 'repeat(2, 1fr)',
          gap: 1.5,
          mb: 2,
        }}
      >
        {QUICK_ACTIONS.map((action) => (
          <Card key={action.path} sx={{ height: 130 }}>
            <CardActionArea sx={{ height: '100%' }} onClick={() => navigate(action.path)}>
              <CardContent sx={{ height: '100%', display: 'flex', flexDirection: 'column', justifyContent: 'space-between' }}>
                <Box color="primary.main">{action.icon}</Box>
                <Box>
                  <Typography variant="subtitle1">{action.title}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    {action.subtitle}
                  </Typography>
                </Box>
              </CardContent>
            </CardActionArea>
          </Card>
        ))}
      </Box>

      <Card>
        <CardActionArea onClick={() => navigate('/about')}>
          <CardContent>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Stack direction="row" spacing={2} alignItems="center">
                <InfoIcon color="primary" />
                <Box>
                  <Typography variant="subtitle1">О приложении</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Версия, ссылки, контакт
                  </Typography>
                </Box>
              </Stack>
              <ArrowForwardIcon color="action" />
            </Stack>
          </CardContent>
        </CardActionArea>
      </Card>
    </Box>
  );
}
