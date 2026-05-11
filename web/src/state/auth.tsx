/* eslint-disable react-refresh/only-export-components */
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  authApi,
  getStoredToken,
  setStoredToken,
  setUnauthorizedHandler,
  usersApi,
} from "../lib/api";
import type { LoginRequest, RegisterRequest, User } from "../lib/types";

const USER_KEY = "worknest.current_user";

interface AuthContextValue {
  user: User | null;
  token: string | null;
  loading: boolean;
  login: (req: LoginRequest) => Promise<void>;
  register: (req: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(() => {
    const raw = localStorage.getItem(USER_KEY);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as User;
    } catch {
      return null;
    }
  });
  const [token, setToken] = useState<string | null>(() => getStoredToken());
  const [loading, setLoading] = useState<boolean>(!!getStoredToken());

  const persistUser = useCallback((u: User | null) => {
    if (u) localStorage.setItem(USER_KEY, JSON.stringify(u));
    else localStorage.removeItem(USER_KEY);
    setUser(u);
  }, []);

  const setAuth = useCallback(
    (u: User, t: string) => {
      setStoredToken(t);
      setToken(t);
      persistUser(u);
    },
    [persistUser],
  );

  const clear = useCallback(() => {
    setStoredToken(null);
    setToken(null);
    persistUser(null);
  }, [persistUser]);

  // On first load, refresh the user from /api/users/me if we have a token —
  // catches stale cached users and surfaces 401s early.
  useEffect(() => {
    if (!token) {
      setLoading(false);
      return;
    }
    let cancelled = false;
    usersApi
      .me()
      .then((u) => {
        if (!cancelled) persistUser(u);
      })
      .catch(() => {
        if (!cancelled) clear();
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [token, persistUser, clear]);

  // Wire the API client to flush auth on 401.
  useEffect(() => {
    setUnauthorizedHandler(() => clear());
    return () => setUnauthorizedHandler(null);
  }, [clear]);

  const login = useCallback(
    async (req: LoginRequest) => {
      const res = await authApi.login(req);
      setAuth(res.user, res.token);
    },
    [setAuth],
  );

  const register = useCallback(
    async (req: RegisterRequest) => {
      const res = await authApi.register(req);
      setAuth(res.user, res.token);
    },
    [setAuth],
  );

  const logout = useCallback(async () => {
    try {
      await authApi.logout();
    } catch {
      /* ignore */
    }
    clear();
  }, [clear]);

  const value = useMemo<AuthContextValue>(
    () => ({ user, token, loading, login, register, logout }),
    [user, token, loading, login, register, logout],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
