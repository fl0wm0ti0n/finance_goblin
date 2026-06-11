-- BUG-0017 / DEC-0106: paired_baseline_id ON DELETE CASCADE

ALTER TABLE forecast_computations
  DROP CONSTRAINT forecast_computations_paired_baseline_id_fkey,
  ADD CONSTRAINT forecast_computations_paired_baseline_id_fkey
    FOREIGN KEY (paired_baseline_id) REFERENCES forecast_computations(id)
    ON DELETE CASCADE NOT VALID;
ALTER TABLE forecast_computations VALIDATE CONSTRAINT forecast_computations_paired_baseline_id_fkey;
