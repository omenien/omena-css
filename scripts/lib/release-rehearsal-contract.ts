export const RELEASE_REHEARSAL_WORKFLOW = "release-rehearsal.yml";
export const RELEASE_REHEARSAL_JOB_ID = "rehearsal";
export const RELEASE_REHEARSAL_JOB_NAME = "release rehearsal";
export const RELEASE_REHEARSAL_ESCALATION_TITLE = "CI failure: Release Rehearsal";
export const RELEASE_REHEARSAL_PATH_STEPS = [
  "Rehearse crate train dry path",
  "Render latest published release notes",
  "Rehearse npm pack dry path",
  "Run release lifecycle checkers",
] as const;
export const RELEASE_REHEARSAL_ENVIRONMENT_STEP = "Check release environment protection";
export const RELEASE_REHEARSAL_ESCALATION_STEP = "Escalate release rehearsal failure";
export const RELEASE_REHEARSAL_RETIREMENT_STEP = "Check release rehearsal retirement ladder";
