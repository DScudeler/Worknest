import { Check } from "lucide-react";
import { useEffect, useState } from "react";

import { Modal } from "../Modal";
import { AGENT_COLORS, AGENT_EMOJIS, defaultColor, defaultEmoji } from "../../lib/agentTemplates";
import { useSavePersona } from "../../lib/agentsHooks";
import {
  AGENT_MODELS,
  CAPABILITIES,
  type AgentModel,
  type Capability,
  type CreatePersonaRequest,
  type Persona,
} from "../../lib/types";

interface Props {
  open: boolean;
  onClose: () => void;
  initial?: Persona | null;
  templates?: Persona[];
}

interface Draft {
  slug: string;
  name: string;
  emoji: string;
  color: string;
  description: string;
  role: string;
  tone: string;
  expertise: string;
  instructions: string;
  capabilities: Capability[];
  model: AgentModel;
  default_cron: string;
}

function blankDraft(): Draft {
  return {
    slug: "",
    name: "",
    emoji: defaultEmoji(),
    color: defaultColor(),
    description: "",
    role: "",
    tone: "",
    expertise: "",
    instructions: "",
    capabilities: ["Comment"],
    model: "Sonnet",
    default_cron: "*/30 * * * *",
  };
}

function fromPersona(p: Persona): Draft {
  return {
    slug: p.slug,
    name: p.name,
    emoji: p.emoji,
    color: p.color,
    description: p.description,
    role: p.role,
    tone: p.tone,
    expertise: p.expertise.join(", "),
    instructions: p.instructions,
    capabilities: [...p.capabilities],
    model: p.model,
    default_cron: p.default_cron,
  };
}

function slugify(name: string): string {
  return name
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 60);
}

export function PersonaEditorModal({ open, onClose, initial, templates = [] }: Props) {
  const isEdit = !!initial;
  const [step, setStep] = useState<"template" | "details">(isEdit ? "details" : "template");
  const [draft, setDraft] = useState<Draft>(() => (initial ? fromPersona(initial) : blankDraft()));
  const save = useSavePersona();

  // Reset whenever the modal is (re)opened or the target persona changes.
  useEffect(() => {
    if (!open) return;
    setStep(isEdit ? "details" : "template");
    setDraft(initial ? fromPersona(initial) : blankDraft());
  }, [open, isEdit, initial]);

  const canSave =
    draft.name.trim().length > 0 &&
    draft.description.trim().length > 0 &&
    draft.instructions.trim().length > 0 &&
    draft.default_cron.trim().split(/\s+/).length === 5;

  const onPickTemplate = (template?: Persona) => {
    if (!template) {
      setDraft(blankDraft());
    } else {
      setDraft(fromPersona({ ...template, slug: "", name: `${template.name} (copy)` }));
    }
    setStep("details");
  };

  const submit = async () => {
    const expertise = draft.expertise
      .split(/[,\n]/)
      .map((s) => s.trim())
      .filter(Boolean);
    if (isEdit && initial) {
      await save.mutateAsync({
        kind: "update",
        id: initial.id,
        data: {
          name: draft.name,
          emoji: draft.emoji,
          color: draft.color,
          description: draft.description,
          role: draft.role,
          tone: draft.tone,
          expertise,
          instructions: draft.instructions,
          capabilities: draft.capabilities,
          model: draft.model,
          default_cron: draft.default_cron,
        },
      });
    } else {
      const slug = draft.slug || slugify(draft.name);
      const req: CreatePersonaRequest = {
        slug,
        name: draft.name,
        emoji: draft.emoji,
        color: draft.color,
        description: draft.description,
        role: draft.role,
        tone: draft.tone,
        expertise,
        instructions: draft.instructions,
        capabilities: draft.capabilities,
        model: draft.model,
        default_cron: draft.default_cron,
      };
      await save.mutateAsync({ kind: "create", data: req });
    }
    onClose();
  };

  const toggleCap = (c: Capability) => {
    setDraft((d) => ({
      ...d,
      capabilities: d.capabilities.includes(c)
        ? d.capabilities.filter((x) => x !== c)
        : [...d.capabilities, c],
    }));
  };

  const title = isEdit ? `Edit ${initial.name}` : "New agent";

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={title}
      size="agents"
      foot={
        step === "template" ? (
          <button type="button" className="btn ghost" onClick={onClose}>
            Cancel
          </button>
        ) : (
          <>
            <button type="button" className="btn ghost" onClick={onClose}>
              Cancel
            </button>
            {!isEdit ? (
              <button
                type="button"
                className="btn secondary"
                onClick={() => setStep("template")}
              >
                Back
              </button>
            ) : null}
            <button
              type="button"
              className="btn primary"
              disabled={!canSave || save.isPending}
              onClick={submit}
            >
              {save.isPending ? "Saving…" : isEdit ? "Save" : "Create agent"}
            </button>
          </>
        )
      }
    >
      {step === "template" ? (
        <div className="tpl-grid">
          <button
            type="button"
            className="tpl-card scratch"
            onClick={() => onPickTemplate(undefined)}
          >
            <div className="tpl-emoji">+</div>
            <div className="tpl-name">From scratch</div>
            <div className="tpl-desc">Empty form. Bring your own persona.</div>
          </button>
          {templates.map((t) => (
            <button
              key={t.id}
              type="button"
              className="tpl-card"
              onClick={() => onPickTemplate(t)}
            >
              <div
                className="tpl-emoji"
                style={{ background: t.color }}
              >
                {t.emoji}
              </div>
              <div className="tpl-name">{t.name}</div>
              <div className="tpl-desc">{t.description}</div>
            </button>
          ))}
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
          <div style={{ display: "flex", gap: 18, alignItems: "flex-start" }}>
            <div className="emoji-pick">
              <div className="field-label" style={{ marginBottom: 8 }}>Avatar</div>
              <div className="emoji-row">
                {AGENT_EMOJIS.map((e) => (
                  <button
                    key={e}
                    type="button"
                    className={`em${draft.emoji === e ? " active" : ""}`}
                    onClick={() => setDraft((d) => ({ ...d, emoji: e }))}
                  >
                    {e}
                  </button>
                ))}
              </div>
              <div className="color-row">
                {AGENT_COLORS.map((c) => (
                  <button
                    key={c}
                    type="button"
                    className={`co${draft.color === c ? " active" : ""}`}
                    style={{ background: c }}
                    onClick={() => setDraft((d) => ({ ...d, color: c }))}
                    aria-label={c}
                  />
                ))}
              </div>
            </div>
            <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: 12 }}>
              <div>
                <div className="field-label">Name</div>
                <input
                  className="input"
                  value={draft.name}
                  onChange={(e) => setDraft((d) => ({ ...d, name: e.target.value }))}
                  placeholder="e.g. Backend Reviewer"
                />
              </div>
              <div>
                <div className="field-label">Description</div>
                <input
                  className="input"
                  value={draft.description}
                  onChange={(e) =>
                    setDraft((d) => ({ ...d, description: e.target.value }))
                  }
                  placeholder="Short blurb that shows on the card"
                />
              </div>
            </div>
          </div>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
            <div>
              <div className="field-label">Role</div>
              <input
                className="input"
                value={draft.role}
                onChange={(e) => setDraft((d) => ({ ...d, role: e.target.value }))}
              />
            </div>
            <div>
              <div className="field-label">Tone</div>
              <input
                className="input"
                value={draft.tone}
                onChange={(e) => setDraft((d) => ({ ...d, tone: e.target.value }))}
              />
            </div>
          </div>

          <div>
            <div className="field-label">Expertise (comma-separated)</div>
            <input
              className="input"
              value={draft.expertise}
              onChange={(e) =>
                setDraft((d) => ({ ...d, expertise: e.target.value }))
              }
            />
          </div>

          <div>
            <div className="field-label">Instructions</div>
            <textarea
              className="textarea"
              rows={6}
              style={{ fontFamily: "ui-monospace, Menlo, monospace" }}
              value={draft.instructions}
              onChange={(e) =>
                setDraft((d) => ({ ...d, instructions: e.target.value }))
              }
            />
          </div>

          <div>
            <div className="field-label">Capabilities</div>
            <div className="cap-pick">
              {CAPABILITIES.map((c) => {
                const on = draft.capabilities.includes(c.id);
                return (
                  <label
                    key={c.id}
                    className={`cap-toggle${on ? " on" : ""}`}
                    onClick={(e) => e.preventDefault()}
                  >
                    <input
                      type="checkbox"
                      checked={on}
                      onChange={() => toggleCap(c.id)}
                    />
                    <span className="cap-check">
                      <Check size={11} />
                    </span>
                    <span>{c.label}</span>
                  </label>
                );
              })}
            </div>
          </div>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
            <div>
              <div className="field-label">Model</div>
              <div className="model-pick">
                {AGENT_MODELS.map((m) => (
                  <button
                    key={m.id}
                    type="button"
                    className={`mp${draft.model === m.id ? " active" : ""}`}
                    onClick={() => setDraft((d) => ({ ...d, model: m.id }))}
                  >
                    <div className="mp-name">{m.label}</div>
                    <div className="mp-tag">{m.tag}</div>
                  </button>
                ))}
              </div>
            </div>
            <div>
              <div className="field-label">Default cron (5-field UTC)</div>
              <input
                className="input"
                value={draft.default_cron}
                onChange={(e) =>
                  setDraft((d) => ({ ...d, default_cron: e.target.value }))
                }
                style={{ fontFamily: "ui-monospace, Menlo, monospace" }}
              />
            </div>
          </div>
        </div>
      )}
    </Modal>
  );
}
