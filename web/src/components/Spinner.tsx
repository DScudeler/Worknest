export function Spinner() {
  return <span className="spinner" aria-label="Loading" />;
}

export function CenterSpinner({ label }: { label?: string }) {
  return (
    <div className="center-page">
      <Spinner />
      {label ? <span>{label}</span> : null}
    </div>
  );
}
