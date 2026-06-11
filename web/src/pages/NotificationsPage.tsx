import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  IconButton,
  Stack,
  Typography,
} from '@mui/material';
import RefreshIcon from '@mui/icons-material/Refresh';
import DoneAllIcon from '@mui/icons-material/DoneAll';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { api } from '../api/client';
import { formatDateTime } from '../utils/date';

const POLL_INTERVAL = 30_000;

export function NotificationsPage() {
  const queryClient = useQueryClient();

  const notificationsQuery = useQuery({
    queryKey: ['notifications'],
    queryFn: () => api.getNotifications(),
    refetchInterval: POLL_INTERVAL,
  });

  const markReadMutation = useMutation({
    mutationFn: (id: string) => api.markNotificationRead(id),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ['notifications'] }),
  });

  const markAllMutation = useMutation({
    mutationFn: () => api.markAllNotificationsRead(),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ['notifications'] }),
  });

  const unreadCount = (notificationsQuery.data ?? []).filter((n) => !n.read).length;

  return (
    <Box p={2} maxWidth={700} mx="auto">
      <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
        <Stack direction="row" spacing={1} alignItems="center">
          <Typography variant="h6">Уведомления</Typography>
          {unreadCount > 0 && <Chip label={unreadCount} size="small" color="primary" />}
        </Stack>
        <Stack direction="row">
          {unreadCount > 0 && (
            <IconButton onClick={() => markAllMutation.mutate()} title="Прочитать все">
              <DoneAllIcon />
            </IconButton>
          )}
          <IconButton onClick={() => void notificationsQuery.refetch()}>
            <RefreshIcon />
          </IconButton>
        </Stack>
      </Stack>

      {notificationsQuery.error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {(notificationsQuery.error as Error).message}
        </Alert>
      )}

      {notificationsQuery.isLoading ? (
        <Box display="flex" justifyContent="center" p={4}>
          <CircularProgress />
        </Box>
      ) : (
        <Stack spacing={1}>
          {(notificationsQuery.data ?? []).map((notification) => (
            <Card
              key={notification.id}
              variant="outlined"
              sx={{ bgcolor: notification.read ? 'background.paper' : 'action.hover' }}
            >
              <CardContent>
                <Stack direction="row" justifyContent="space-between" alignItems="flex-start">
                  <Box flex={1}>
                    <Typography variant="subtitle1">
                      {notification.title ?? 'Уведомление'}
                    </Typography>
                    {notification.message && (
                      <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
                        {notification.message}
                      </Typography>
                    )}
                    <Typography variant="caption" color="text.secondary" display="block" mt={1}>
                      {formatDateTime(notification.created_at)}
                    </Typography>
                  </Box>
                  {!notification.read && (
                    <Button
                      size="small"
                      onClick={() => markReadMutation.mutate(notification.id)}
                      disabled={markReadMutation.isPending}
                    >
                      Прочитано
                    </Button>
                  )}
                </Stack>
              </CardContent>
            </Card>
          ))}
          {(notificationsQuery.data ?? []).length === 0 && (
            <Typography color="text.secondary" textAlign="center" py={4}>
              Нет уведомлений
            </Typography>
          )}
        </Stack>
      )}
    </Box>
  );
}
