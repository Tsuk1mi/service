import {
  Badge,
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
import { useQuery } from '@tanstack/react-query';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { api } from '../api/client';

const NAV_ITEMS = [
  { path: '/', label: 'Главная', shortLabel: 'Главная', icon: <HomeIcon />, key: 'home' },
  { path: '/profile', label: 'Профиль', shortLabel: 'Профиль', icon: <PersonIcon />, key: 'profile' },
  { path: '/blocks', label: 'Мои блокировки', shortLabel: 'Блокировки', icon: <ListIcon />, key: 'blocks' },
  { path: '/blocked-by', label: 'Меня заблокировали', shortLabel: 'Заблокировали', icon: <WarningIcon />, key: 'blocked-by' },
  { path: '/notifications', label: 'Уведомления', shortLabel: 'Уведомления', icon: <NotificationsIcon />, key: 'notifications' },
];

const SIDEBAR_EXTRA = [
  { path: '/about', label: 'О приложении', icon: <InfoIcon /> },
];

const POLL_INTERVAL = 30_000;

export function AppLayout() {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const location = useLocation();
  const navigate = useNavigate();

  const notificationsQuery = useQuery({
    queryKey: ['notifications'],
    queryFn: () => api.getNotifications(),
    refetchInterval: POLL_INTERVAL,
  });
  const unreadCount = (notificationsQuery.data ?? []).filter((n) => !n.read).length;

  const currentIndex = NAV_ITEMS.findIndex((item) =>
    item.path === '/' ? location.pathname === '/' : location.pathname.startsWith(item.path),
  );

  const navIcon = (item: (typeof NAV_ITEMS)[number]) => {
    if (item.key === 'notifications' && unreadCount > 0) {
      return (
        <Badge badgeContent={unreadCount} color="error" max={99}>
          {item.icon}
        </Badge>
      );
    }
    return item.icon;
  };

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
                <ListItemIcon>
                  {'key' in item && item.key === 'notifications' && unreadCount > 0 ? (
                    <Badge badgeContent={unreadCount} color="error" max={99}>
                      {item.icon}
                    </Badge>
                  ) : (
                    item.icon
                  )}
                </ListItemIcon>
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
              <Typography variant="h6" color="primary" fontWeight={700} sx={{ flex: 1 }}>
                Rimskiy
              </Typography>
              {unreadCount > 0 && (
                <Badge
                  badgeContent={unreadCount}
                  color="error"
                  max={99}
                  sx={{ cursor: 'pointer' }}
                  onClick={() => navigate('/notifications')}
                >
                  <NotificationsIcon color="action" />
                </Badge>
              )}
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
                icon={navIcon(item)}
              />
            ))}
          </BottomNavigation>
        )}
      </Box>
    </Box>
  );
}
