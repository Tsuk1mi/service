import {
  BottomNavigation,
  BottomNavigationAction,
  Box,
  Drawer,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Toolbar,
  AppBar,
  Typography,
  useMediaQuery,
  useTheme,
} from '@mui/material';
import HomeIcon from '@mui/icons-material/Home';
import PersonIcon from '@mui/icons-material/Person';
import ListIcon from '@mui/icons-material/List';
import WarningIcon from '@mui/icons-material/Warning';
import NotificationsIcon from '@mui/icons-material/Notifications';
import InfoIcon from '@mui/icons-material/Info';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';

const NAV_ITEMS = [
  { path: '/', label: 'Главная', shortLabel: 'Главная', icon: <HomeIcon /> },
  { path: '/profile', label: 'Профиль', shortLabel: 'Профиль', icon: <PersonIcon /> },
  { path: '/blocks', label: 'Мои блокировки', shortLabel: 'Блокировки', icon: <ListIcon /> },
  { path: '/blocked-by', label: 'Меня заблокировали', shortLabel: 'Заблокировали', icon: <WarningIcon /> },
  { path: '/notifications', label: 'Уведомления', shortLabel: 'Уведомления', icon: <NotificationsIcon /> },
];

const SIDEBAR_EXTRA = [
  { path: '/about', label: 'О приложении', icon: <InfoIcon /> },
];

export function AppLayout() {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const location = useLocation();
  const navigate = useNavigate();

  const currentIndex = NAV_ITEMS.findIndex((item) =>
    item.path === '/' ? location.pathname === '/' : location.pathname.startsWith(item.path),
  );

  return (
    <Box sx={{ display: 'flex', minHeight: '100vh', bgcolor: 'background.default' }}>
      {!isMobile && (
        <Drawer
          variant="permanent"
          sx={{
            width: 240,
            flexShrink: 0,
            '& .MuiDrawer-paper': { width: 240, boxSizing: 'border-box' },
          }}
        >
          <Toolbar>
            <Typography variant="h6" color="primary" fontWeight={700}>
              Rimskiy
            </Typography>
          </Toolbar>
          <List>
            {[...NAV_ITEMS, ...SIDEBAR_EXTRA].map((item) => (
              <ListItemButton
                key={item.path}
                selected={
                  item.path === '/'
                    ? location.pathname === '/'
                    : location.pathname.startsWith(item.path)
                }
                onClick={() => navigate(item.path)}
              >
                <ListItemIcon>{item.icon}</ListItemIcon>
                <ListItemText primary={item.label} />
              </ListItemButton>
            ))}
          </List>
        </Drawer>
      )}

      <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        {isMobile && (
          <AppBar position="sticky" color="default" elevation={1}>
            <Toolbar>
              <Typography variant="h6" color="primary" fontWeight={700}>
                Rimskiy
              </Typography>
            </Toolbar>
          </AppBar>
        )}

        <Box component="main" sx={{ flex: 1, pb: isMobile ? 7 : 0, overflow: 'auto' }}>
          <Outlet />
        </Box>

        {isMobile && (
          <BottomNavigation
            showLabels
            value={currentIndex >= 0 ? currentIndex : false}
            onChange={(_, value) => navigate(NAV_ITEMS[value].path)}
            sx={{ position: 'fixed', bottom: 0, left: 0, right: 0, borderTop: 1, borderColor: 'divider' }}
          >
            {NAV_ITEMS.map((item) => (
              <BottomNavigationAction
                key={item.path}
                label={item.shortLabel}
                icon={item.icon}
              />
            ))}
          </BottomNavigation>
        )}
      </Box>
    </Box>
  );
}
