import { useEffect, useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  FormControlLabel,
  IconButton,
  Stack,
  Switch,
  TextField,
  Typography,
} from '@mui/material';
import DeleteIcon from '@mui/icons-material/Delete';
import StarIcon from '@mui/icons-material/Star';
import StarBorderIcon from '@mui/icons-material/StarBorder';
import AddIcon from '@mui/icons-material/Add';
import RefreshIcon from '@mui/icons-material/Refresh';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import { useAuth } from '../auth/AuthContext';
import { TimePickerDialog } from '../components/TimePickerDialog';
import { formatPlate, normalizePlate, validatePlate } from '../utils/plate';
import { formatPhone, normalizePhone, validatePhone } from '../utils/phone';
import { formatTime } from '../utils/date';

export function ProfilePage() {
  const { logout } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [name, setName] = useState('');
  const [phone, setPhone] = useState('');
  const [telegram, setTelegram] = useState('');
  const [showContacts, setShowContacts] = useState(true);
  const [ownerType, setOwnerType] = useState('');
  const [newPlate, setNewPlate] = useState('');
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [timePickerPlateId, setTimePickerPlateId] = useState<string | null>(null);
  const [timePickerInitial, setTimePickerInitial] = useState('');

  const profileQuery = useQuery({
    queryKey: ['profile'],
    queryFn: () => api.getProfile(),
  });

  const platesQuery = useQuery({
    queryKey: ['user-plates'],
    queryFn: () => api.getUserPlates(),
  });

  const profile = profileQuery.data;

  useEffect(() => {
    if (profile) {
      setName(profile.name ?? '');
      setPhone(profile.phone ?? '');
      setTelegram(profile.telegram ?? '');
      setShowContacts(profile.show_contacts);
      setOwnerType(profile.owner_type ?? '');
    }
  }, [profile]);

  const updateProfileMutation = useMutation({
    mutationFn: () =>
      api.updateProfile({
        name: name || null,
        phone: validatePhone(phone) ? normalizePhone(phone) : null,
        telegram: telegram || null,
        show_contacts: showContacts,
        owner_type: ownerType || null,
      }),
    onSuccess: () => {
      setMessage('Профиль сохранён');
      setError(null);
      void queryClient.invalidateQueries({ queryKey: ['profile'] });
    },
    onError: (e: Error) => setError(e.message),
  });

  const createPlateMutation = useMutation({
    mutationFn: (plate: string) =>
      api.createUserPlate({ plate: normalizePlate(plate), is_primary: (platesQuery.data?.length ?? 0) === 0 }),
    onSuccess: () => {
      setNewPlate('');
      setMessage('Автомобиль добавлен');
      void queryClient.invalidateQueries({ queryKey: ['user-plates', 'profile'] });
    },
    onError: (e: Error) => setError(e.message),
  });

  const deletePlateMutation = useMutation({
    mutationFn: (id: string) => api.deleteUserPlate(id),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ['user-plates', 'profile'] }),
    onError: (e: Error) => setError(e.message),
  });

  const setPrimaryMutation = useMutation({
    mutationFn: (id: string) => api.setPrimaryPlate(id),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ['user-plates', 'profile'] }),
  });

  const updateDepartureMutation = useMutation({
    mutationFn: ({ id, time }: { id: string; time: string | null }) =>
      api.updateUserPlate(id, time),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ['user-plates'] }),
  });

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  if (profileQuery.isLoading) {
    return (
      <Box display="flex" justifyContent="center" p={4}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box p={2} maxWidth={700} mx="auto">
      <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
        <Typography variant="h6">Профиль</Typography>
        <Button color="error" onClick={handleLogout}>
          Выйти
        </Button>
      </Stack>

      {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}
      {message && <Alert severity="success" sx={{ mb: 2 }} onClose={() => setMessage(null)}>{message}</Alert>}

      <Card sx={{ mb: 2 }}>
        <CardContent>
          <Typography variant="subtitle1" gutterBottom>
            Контактная информация
          </Typography>
          <Stack spacing={2}>
            <TextField label="Имя" value={name} onChange={(e) => setName(e.target.value)} fullWidth />
            <TextField
              label="Телефон"
              value={phone}
              onChange={(e) => setPhone(e.target.value)}
              placeholder={formatPhone('+79991234567')}
              fullWidth
            />
            <TextField
              label="Telegram"
              value={telegram}
              onChange={(e) => setTelegram(e.target.value)}
              placeholder="@username"
              fullWidth
            />
            <FormControlLabel
              control={<Switch checked={showContacts} onChange={(e) => setShowContacts(e.target.checked)} />}
              label="Показывать контакты другим пользователям"
            />
            <TextField
              select
              label="Тип владельца"
              value={ownerType}
              onChange={(e) => setOwnerType(e.target.value)}
              fullWidth
              SelectProps={{ native: true }}
            >
              <option value="">Не указано</option>
              <option value="owner">Владелец</option>
              <option value="renter">Арендатор</option>
            </TextField>
            <Button
              variant="contained"
              onClick={() => updateProfileMutation.mutate()}
              disabled={updateProfileMutation.isPending}
            >
              Сохранить профиль
            </Button>
          </Stack>
        </CardContent>
      </Card>

      <Card>
        <CardContent>
          <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
            <Typography variant="subtitle1">Мои автомобили</Typography>
            <IconButton onClick={() => void platesQuery.refetch()} size="small">
              <RefreshIcon />
            </IconButton>
          </Stack>

          <Stack direction="row" spacing={1} mb={2}>
            <TextField
              label="Новый номер"
              value={newPlate}
              onChange={(e) => setNewPlate(e.target.value.toUpperCase())}
              size="small"
              fullWidth
              placeholder="А123БВ777"
            />
            <Button
              variant="contained"
              startIcon={<AddIcon />}
              onClick={() => {
                if (!validatePlate(newPlate)) {
                  setError('Некорректный номер автомобиля');
                  return;
                }
                createPlateMutation.mutate(newPlate);
              }}
              disabled={createPlateMutation.isPending}
            >
              Добавить
            </Button>
          </Stack>

          <Stack spacing={1}>
            {(platesQuery.data ?? []).map((plate) => (
              <Card key={plate.id} variant="outlined">
                <CardContent sx={{ py: 1.5, '&:last-child': { pb: 1.5 } }}>
                  <Stack direction="row" alignItems="center" justifyContent="space-between">
                    <Box>
                      <Stack direction="row" spacing={1} alignItems="center">
                        <Typography fontWeight={600}>{formatPlate(plate.plate)}</Typography>
                        {plate.is_primary && <Chip label="Основной" size="small" color="primary" />}
                      </Stack>
                      <Typography variant="caption" color="text.secondary">
                        Выезд: {formatTime(plate.departure_time)}
                        {' · '}
                        <Button
                          size="small"
                          sx={{ minWidth: 0, p: 0, verticalAlign: 'baseline' }}
                          onClick={() => {
                            setTimePickerPlateId(plate.id);
                            setTimePickerInitial(plate.departure_time ?? '');
                          }}
                        >
                          изменить
                        </Button>
                      </Typography>
                    </Box>
                    <Stack direction="row">
                      <IconButton
                        size="small"
                        onClick={() => setPrimaryMutation.mutate(plate.id)}
                        disabled={plate.is_primary}
                      >
                        {plate.is_primary ? <StarIcon color="primary" /> : <StarBorderIcon />}
                      </IconButton>
                      <IconButton
                        size="small"
                        color="error"
                        onClick={() => deletePlateMutation.mutate(plate.id)}
                      >
                        <DeleteIcon />
                      </IconButton>
                    </Stack>
                  </Stack>
                </CardContent>
              </Card>
            ))}
            {(platesQuery.data ?? []).length === 0 && (
              <Typography color="text.secondary">Добавьте хотя бы один автомобиль</Typography>
            )}
          </Stack>
        </CardContent>
      </Card>

      <TimePickerDialog
        open={timePickerPlateId !== null}
        initialTime={timePickerInitial}
        onClose={() => setTimePickerPlateId(null)}
        onConfirm={(time) => {
          if (timePickerPlateId) {
            updateDepartureMutation.mutate({ id: timePickerPlateId, time });
          }
          setTimePickerPlateId(null);
        }}
      />
    </Box>
  );
}
