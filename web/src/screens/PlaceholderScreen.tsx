import { Topbar } from "../components/Topbar";

export function PlaceholderScreen({ title }: { title: string }) {
  return (
    <>
      <Topbar crumbs={[{ label: "Workspace" }, { label: title }]} />
      <div className="content center-page" style={{ minHeight: 400 }}>
        <h2 style={{ margin: 0 }}>{title}</h2>
        <p className="muted">Coming soon.</p>
      </div>
    </>
  );
}
