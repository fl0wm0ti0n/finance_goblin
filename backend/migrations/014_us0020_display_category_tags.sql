-- US-0020: majority display category overlay + operator tags (DEC-0100, DEC-0101)

ALTER TABLE subscription_patterns
    ADD COLUMN display_category_id TEXT NULL;

CREATE TABLE operator_tags (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE subscription_pattern_tags (
    pattern_id UUID NOT NULL REFERENCES subscription_patterns(id) ON DELETE CASCADE,
    tag_id     UUID NOT NULL REFERENCES operator_tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (pattern_id, tag_id)
);

CREATE INDEX idx_subscription_pattern_tags_tag ON subscription_pattern_tags(tag_id);
