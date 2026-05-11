import { useState } from "react";
import { Link, Navigate } from "react-router-dom";
import { useAuth } from "../state/auth";
import { ThemeToggle } from "../components/ThemeToggle";
import { ApiError } from "../lib/api";

export function RegisterScreen() {
  const { register, user, loading } = useAuth();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (loading) return null;
  if (user) return <Navigate to="/" replace />;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await register({ username, email, password });
    } catch (err) {
      if (err instanceof ApiError) setError(err.message);
      else setError("Sign-up failed. Please try again.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-art">
        <div className="blob b1" />
        <div className="blob b2" />
        <div className="login-logo">
          <span className="logo-mark">W</span>
          <span>Worknest</span>
        </div>
        <div className="quote">
          <h2>Welcome aboard.</h2>
          <p>Create your workspace and start moving work forward in minutes.</p>
        </div>
      </div>
      <div className="login-form-wrap">
        <form className="login-form" onSubmit={handleSubmit}>
          <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: 20 }}>
            <ThemeToggle />
          </div>
          <h1>Create your account</h1>
          <p className="sub">Free for now — no card needed.</p>
          {error ? <div className="err">{error}</div> : null}
          <div className="field">
            <label className="field-label" htmlFor="reg-username">Username</label>
            <input
              id="reg-username"
              className="input"
              type="text"
              autoComplete="username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              minLength={3}
              maxLength={50}
              required
            />
          </div>
          <div className="field">
            <label className="field-label" htmlFor="reg-email">Email</label>
            <input
              id="reg-email"
              className="input"
              type="email"
              autoComplete="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </div>
          <div className="field">
            <label className="field-label" htmlFor="reg-password">Password</label>
            <input
              id="reg-password"
              className="input"
              type="password"
              autoComplete="new-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              minLength={8}
              required
            />
          </div>
          <button className="btn primary" type="submit" disabled={busy}>
            {busy ? "Creating account…" : "Create account"}
          </button>
          <div className="alt">
            Already have an account? <Link to="/login">Sign in</Link>
          </div>
        </form>
      </div>
    </div>
  );
}
