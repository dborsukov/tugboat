/**
 * Exists as one place to store all module ids and to avoid cicrular import that happens
 * when a module that offers options needs to read its own id. It cannot read id
 * from registry because registry imports the module causing an infinite cycle.
 */
export const REVENUE_ID = "revenue";
export const OUTSTANDING_ID = "outstanding";
export const IN_PROGRESS_ID = "in-progress";
export const VAT_DUE_ID = "vat-due";
export const GROSS_INCOME_ID = "gross-income";
export const TOP_SERVICES_ID = "top-services";
export const CLIENT_MIX_ID = "client-mix";
export const FINANCIAL_RESULT_ID = "financial-result";
