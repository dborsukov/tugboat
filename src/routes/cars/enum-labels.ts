import type { DriveType, FuelType } from "$lib/bindings";

export const fuelTypeLabels: Record<FuelType, string> = {
  Gasoline: "Бензин",
  Diesel: "Дизель",
  LpgCng: "Газ (LPG/CNG)",
  Hybrid: "Гібрид",
  Electric: "Електро",
};

export const driveTypeLabels: Record<DriveType, string> = {
  FrontWheel: "Передній привід",
  RearWheel: "Задній привід",
  AllWheel: "Повний привід (AWD)",
  FourWheel: "Повний привід (4WD)",
};
