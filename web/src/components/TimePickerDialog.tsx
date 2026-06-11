import {
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Button,
  TextField,
  Stack,
} from '@mui/material';
import { useEffect, useState } from 'react';

interface TimePickerDialogProps {
  open: boolean;
  initialTime?: string;
  title?: string;
  onClose: () => void;
  onConfirm: (time: string) => void;
}

export function TimePickerDialog({
  open,
  initialTime = '',
  title = 'Время выезда',
  onClose,
  onConfirm,
}: TimePickerDialogProps) {
  const [time, setTime] = useState(initialTime);

  useEffect(() => {
    if (open) setTime(initialTime);
  }, [open, initialTime]);

  return (
    <Dialog open={open} onClose={onClose} maxWidth="xs" fullWidth>
      <DialogTitle>{title}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ mt: 1 }}>
          <TextField
            label="Время (ЧЧ:ММ)"
            type="time"
            value={time}
            onChange={(e) => setTime(e.target.value)}
            InputLabelProps={{ shrink: true }}
            fullWidth
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Отмена</Button>
        <Button variant="contained" onClick={() => onConfirm(time)} disabled={!time}>
          Сохранить
        </Button>
      </DialogActions>
    </Dialog>
  );
}
