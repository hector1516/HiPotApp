// Pasos del modo automático. Editar aquí cambia el stepper, los prompts y
// los números de paso que usa App.tsx.
export type AutoStepId = 'model' | 'serial' | 'batch' | 'hipot' | 'plc' | 'result';

export type AutoStep = {
  id: AutoStepId;
  label: string;
  prompt: string;
  description: string;
};

export const AUTO_STEPS: AutoStep[] = [
  {
    id: 'model',
    label: 'Modelo',
    prompt: 'Selecciona el modelo',
    description: 'Elige la receta de la lista. Se recuerda el último modelo usado.',
  },
  {
    id: 'serial',
    label: 'Serie',
    prompt: 'Escanea la serie',
    description: 'Identifica la unidad bajo prueba.',
  },
  {
    id: 'batch',
    label: 'Batch',
    prompt: 'Escanea tu batch para iniciar',
    description: 'Crea la ejecución y arranca la prueba.',
  },
  {
    id: 'hipot',
    label: 'Hipot',
    prompt: 'Prueba Hipot en curso',
    description: 'El Hipot mide aislamiento y detecta cortos.',
  },
  {
    id: 'plc',
    label: 'Eléctrica',
    prompt: 'Prueba eléctrica en curso',
    description: 'El PLC mide consumo por componente.',
  },
  {
    id: 'result',
    label: 'Resultado',
    prompt: 'Resultado final',
    description: 'GOOD/BAD, etiqueta y siguiente unidad.',
  },
];

export const AUTO_STEP_COUNT = AUTO_STEPS.length;

export function stepNumber(id: AutoStepId): number {
  return AUTO_STEPS.findIndex(s => s.id === id) + 1;
}

export function stepAt(index: number): AutoStep {
  return AUTO_STEPS[Math.max(0, Math.min(index, AUTO_STEP_COUNT - 1))];
}
