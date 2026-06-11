import { useRef, useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  IconButton,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import PhotoCameraIcon from '@mui/icons-material/PhotoCamera';
import PhoneIcon from '@mui/icons-material/Phone';
import RefreshIcon from '@mui/icons-material/Refresh';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { api } from '../api/client';
import type { Block } from '../api/types';
import { TimePickerDialog } from '../components/TimePickerDialog';
import { formatDateTime } from '../utils/date';
import { formatPlate, normalizePlate, validatePlate } from '../utils/plate';

export function MyBlocksPage() {
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [newPlate, setNewPlate] = useState('');
  const [departureTime, setDepartureTime] = useState('');
  const notifyOwner = true;
  const [error, setError] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<Block | null>(null);
  const [showTimePicker, setShowTimePicker] = useState(false);
  const [isRecognizing, setIsRecognizing] = useState(false);

  const profileQuery = useQuery({
    queryKey: ['profile'],
    queryFn: () => api.getProfile(),
  });

  const blocksQuery = useQuery({
    queryKey: ['my-blocks'],
    queryFn: () => api.getMyBlocks(),
  });

  const createMutation = useMutation({
    mutationFn: () =>
      api.createBlock({
        blocked_plate: normalizePlate(newPlate),
        notify_owner: notifyOwner,
        departure_time: departureTime || profileQuery.data?.departure_time || null,
        notification_method: 'telegram',
      }),
    onSuccess: () => {
      setNewPlate('');
      setError(null);
      void queryClient.invalidateQueries({ queryKey: ['my-blocks'] });
    },
    onError: (e: Error) => setError(e.message),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteBlock(id),
    onSuccess: () => {
      setDeleteTarget(null);
      void queryClient.invalidateQueries({ queryKey: ['my-blocks'] });
    },
    onError: (e: Error) => setError(e.message),
  });

  const warnMutation = useMutation({
    mutationFn: (id: string) => api.warnOwner(id),
    onSuccess: () => setError(null),
    onError: (e: Error) => setError(e.message),
  });

  const handleCreate = () => {
    if (!validatePlate(newPlate)) {
      setError('Некорректный номер автомобиля');
      return;
    }
    createMutation.mutate();
  };

  const handleOcr = async (file: File) => {
    setIsRecognizing(true);
    setError(null);
    try {
      const result = await api.recognizePlateFromImage(file);
      if (result.success && result.plate) {
        setNewPlate(result.plate);
      } else {
        setError(result.error ?? 'Не удалось распознать номер');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Ошибка OCR');
    } finally {
      setIsRecognizing(false);
    }
  };

  return (
    <Box p={2} maxWidth={700} mx="auto">
      <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
        <Typography variant="h6">Мои блокировки</Typography>
        <IconButton onClick={() => void blocksQuery.refetch()}>
          <RefreshIcon />
        </IconButton>
      </Stack>

      {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}

      <Card sx={{ mb: 2 }}>
        <CardContent>
          <Stack direction="row" spacing={1} alignItems="center" mb={2}>
            <AddIcon color="primary" />
            <Typography variant="subtitle1">Добавить блокировку</Typography>
          </Stack>
          <Stack spacing={2}>
            <TextField
              label="Номер автомобиля"
              value={newPlate}
              onChange={(e) => setNewPlate(e.target.value.toUpperCase())}
              fullWidth
              placeholder="А123БВ777"
            />
            <Stack direction="row" spacing={1}>
              <Button
                variant="outlined"
                startIcon={isRecognizing ? <CircularProgress size={18} /> : <PhotoCameraIcon />}
                onClick={() => fileInputRef.current?.click()}
                disabled={isRecognizing}
              >
                Распознать с фото
              </Button>
              <input
                ref={fileInputRef}
                type="file"
                accept="image/*"
                hidden
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  if (file) void handleOcr(file);
                  e.target.value = '';
                }}
              />
              <Button variant="text" onClick={() => setShowTimePicker(true)}>
                Время выезда: {departureTime || profileQuery.data?.departure_time || '—'}
              </Button>
            </Stack>
            <Button
              variant="contained"
              fullWidth
              onClick={handleCreate}
              disabled={createMutation.isPending}
            >
              {createMutation.isPending ? <CircularProgress size={24} /> : 'Заблокировать'}
            </Button>
          </Stack>
        </CardContent>
      </Card>

      {blocksQuery.isLoading ? (
        <Box display="flex" justifyContent="center" p={4}>
          <CircularProgress />
        </Box>
      ) : (
        <Stack spacing={1}>
          {(blocksQuery.data ?? []).map((block) => (
            <Card key={block.id} variant="outlined">
              <CardContent>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Box>
                    <Typography fontWeight={600}>{formatPlate(block.blocked_plate)}</Typography>
                    <Typography variant="caption" color="text.secondary">
                      {formatDateTime(block.created_at)}
                    </Typography>
                  </Box>
                  <Stack direction="row">
                    <IconButton
                      color="primary"
                      onClick={() => warnMutation.mutate(block.id)}
                      disabled={warnMutation.isPending}
                      title="Предупредить владельца"
                    >
                      <PhoneIcon />
                    </IconButton>
                    <IconButton color="error" onClick={() => setDeleteTarget(block)}>
                      <DeleteIcon />
                    </IconButton>
                  </Stack>
                </Stack>
              </CardContent>
            </Card>
          ))}
          {(blocksQuery.data ?? []).length === 0 && (
            <Typography color="text.secondary" textAlign="center" py={4}>
              Нет активных блокировок
            </Typography>
          )}
        </Stack>
      )}

      <Dialog open={deleteTarget !== null} onClose={() => setDeleteTarget(null)}>
        <DialogTitle>Удалить блокировку?</DialogTitle>
        <DialogContent>
          <Typography>
            Снять блокировку с {deleteTarget ? formatPlate(deleteTarget.blocked_plate) : ''}?
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteTarget(null)}>Отмена</Button>
          <Button
            color="error"
            variant="contained"
            onClick={() => deleteTarget && deleteMutation.mutate(deleteTarget.id)}
          >
            Удалить
          </Button>
        </DialogActions>
      </Dialog>

      <TimePickerDialog
        open={showTimePicker}
        initialTime={departureTime || profileQuery.data?.departure_time || ''}
        onClose={() => setShowTimePicker(false)}
        onConfirm={(time) => {
          setDepartureTime(time);
          setShowTimePicker(false);
        }}
      />
    </Box>
  );
}
