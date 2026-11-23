import React from 'react';
import { usePLC } from '../hooks/usePLC';
import { useHipot } from '../hooks/useHipot';

/** Estado compacto de equipos para el pie de la app (siempre visible). */
export function EquipmentStatus({
  hardwareReady,
  hasRecipe,
}: {
  hardwareReady: boolean;
  hasRecipe: boolean;
}) {
  const plc = usePLC(hardwareReady);
  const hipot = useHipot(hardwareReady);

  const stateClass = (on: boolean | null) =>
    on === true ? 'ok' : on === false ? 'bad' : '';

  return (
    <div className="footer-equipment" title="Estado de los equipos">
      <span className="footer-equipment-label">Equipos</span>
      <span className="equip-item">
        <span className={`dot ${stateClass(plc.connected)}`} />
        PLC
      </span>
      <span className="equip-item">
        <span className={`dot ${stateClass(hipot.connected)}`} />
        Hipot
      </span>
      <span className="equip-item">
        <span className={`dot ${stateClass(hasRecipe ? true : null)}`} />
        Receta
      </span>
    </div>
  );
}
