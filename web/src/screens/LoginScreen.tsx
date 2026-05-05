import { useState } from "react";
import { Link, Navigate, useLocation } from "react-router-dom";
import { useAuth } from "../state/auth";
import { ThemeToggle } from "../components/ThemeToggle";
import { ApiError } from "../lib/api";
import { StatusPill } from "../components/StatusPill";

export function LoginScreen() {
  const { login, user, loading } = useAuth();
  const location = useLocation();
  const [identifier, setIdentifier] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (loading) return null;
  if (user) {
    const from = (location.state as { from?: { pathname?: string } } | null)?.from?.pathname;
    return <Navigate to={from && from !== "/login" ? from : "/"} replace />;
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await login({ username: identifier, password });
    } catch (err) {
      if (err instanceof ApiError) setError(err.message);
      else setError("Sign-in failed. Please try again.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="login-page" data-screen-label="Login">
      <div className="login-art">
        <div className="blob b1" />
        <div className="blob b2" />
        <div className="login-logo">
          <span className="logo-mark">W</span>
          <span>Worknest</span>
        </div>
        <div className="quote">
          <h2>Where work finds its rhythm.</h2>
          <p>Tickets, projects, and the people doing the work — all in one calm place.</p>
        </div>
        <div className="preview-tickets">
          <div className="preview-ticket">
            <span className="pt-id">WEB-142</span>
            <span className="pt-title">New pricing page hero</span>
            <StatusPill status="InProgress" />
          </div>
          <div className="preview-ticket" style={{ marginLeft: 30 }}>
            <span className="pt-id">MOB-073</span>
            <span className="pt-title">Onboarding screens</span>
            <StatusPill status="Open" />
          </div>
          <div className="preview-ticket" style={{ marginLeft: 60 }}>
            <span className="pt-id">DSGN-12</span>
            <span className="pt-title">Token sync to Figma</span>
            <StatusPill status="Done" />
          </div>
        </div>
      </div>
      <div className="login-form-wrap">
        <form className="login-form" onSubmit={handleSubmit}>
          <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: 20 }}>
            <ThemeToggle />
          </div>
          <h1>Welcome back</h1>
          <p className="sub">Sign in to your Worknest workspace.</p>
          {error ? <div className="err">{error}</div> : null}
          <div className="field">
            <label className="field-label" htmlFor="login-identifier">
              Username or email
            </label>
            <input
              id="login-identifier"
              className="input"
              type="text"
              autoComplete="username"
              value={identifier}
              onChange={(e) => setIdentifier(e.target.value)}
              placeholder="you@company.com"
              required
            />
          </div>
          <div className="field">
            <label
              className="field-label"
              htmlFor="login-password"
              style={{ display: "flex", justifyContent: "space-between" }}
            >
              <span>Password</span>
            </label>
            <input
              id="login-password"
              className="input"
              type="password"
              autoComplete="current-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </div>
          <button className="btn primary" type="submit" disabled={busy}>
            {busy ? "Signing in…" : "Sign in"}
          </button>
          <div className="divider">or continue with</div>
          <div className="sso-row">
            <button
              type="button"
              className="sso-btn"
              disabled
              title="SSO coming soon"
            >
              Google
            </button>
            <button
              type="button"
              className="sso-btn"
              disabled
              title="SSO coming soon"
            >
              GitHub
            </button>
          </div>
          <div className="alt">
            New here? <Link to="/register">Create an account</Link>
          </div>
        </form>
      </div>
    </div>
  );
}
