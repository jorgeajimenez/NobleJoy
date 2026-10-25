"""MuJoCo Physics Engine Simulation & Robotics Sanity Check Gate (`mujoco_sim.py`).

Runs the real MuJoCo (`mujoco`) 3D rigid-body vehicle + suspension + wheel friction +
arm kinematics simulation for the driving game and streams body transforms & physics
verification telemetry to the Rust Bevy client.
"""

import math
from pathlib import Path
from typing import Any, Dict, List, Optional

from agents.artifact_puller import write_rover_urdf
from agents.mujoco_builder import build_mujoco_car_mjcf


class MujocoDrivingSession:
    """Manages an active MuJoCo MJCF simulation for a commissioned driving level."""

    def __init__(self, project_id: str, car_spec: Dict[str, Any], level_entities: List[Dict[str, Any]]) -> None:
        self.project_id = project_id
        self.car_spec = dict(car_spec)
        self.level_entities = level_entities
        self.mjcf_path = build_mujoco_car_mjcf(project_id, self.car_spec)
        self.urdf_path = write_rover_urdf(project_id, self.car_spec)

        self.upgrade_equipped: bool = False
        self.arm_extension_m: float = 0.25
        self.payload_mass_kg: float = 0.35

        # Analytical backup state if MuJoCo C-runtime isn't loaded yet
        self.pos_x: float = 0.0
        self.pos_y: float = 0.0
        self.pos_z: float = float(self.car_spec.get("ground_clearance_m", 0.18)) + 0.15
        self.yaw: float = 0.785  # facing (10, 10)
        self.speed: float = 0.0
        self.wheel_spin: float = 0.0

        self._mj_model: Any = None
        self._mj_data: Any = None
        self._init_mujoco()

    def _init_mujoco(self) -> None:
        """Load the MJCF XML into native MuJoCo if installed."""
        try:
            import mujoco

            self._mj_model = mujoco.MjModel.from_xml_path(self.mjcf_path)
            self._mj_data = mujoco.MjData(self._mj_model)
            mujoco.mj_forward(self._mj_model, self._mj_data)
            print(f"[MuJoCo] Initialized native MjModel from {self.mjcf_path}")
        except Exception as exc:
            print(f"[MuJoCo] Running physics-aware analytical solver ({exc})")
            self._mj_model = None
            self._mj_data = None

    def apply_physics_hotfix(self, hotfix_params: Dict[str, Any]) -> None:
        """Hot-reload MuJoCo vehicle parameters (friction, motor torque, clearance) from Visual QA agent."""
        self.car_spec.update(hotfix_params)
        self.mjcf_path = build_mujoco_car_mjcf(self.project_id, self.car_spec)
        self.urdf_path = write_rover_urdf(self.project_id, self.car_spec)
        self._init_mujoco()

    def compute_physics_sanity_check(self) -> Dict[str, Any]:
        """Run Gretchen's Verification Gate: Center-of-Mass (CoM) footprint & Torque (tau = r x F)."""
        chassis_mass = float(self.car_spec.get("chassis_mass_kg", 4.5))
        wheelbase_half_x = 0.45
        track_half_y = 0.45

        # Combined Center of Mass shift when modular arm/upgrade extends forward
        total_mass = chassis_mass + (self.payload_mass_kg if self.upgrade_equipped else 0.15)
        com_offset_x = (self.arm_extension_m * self.payload_mass_kg) / total_mass
        com_stable = abs(com_offset_x) < wheelbase_half_x

        # Torque validation: tau = r * F (converted to kg*cm for hobby/industrial servo comparison)
        lever_arm_cm = self.arm_extension_m * 100.0
        required_torque_kg_cm = round(lever_arm_cm * self.payload_mass_kg, 2)
        base_servo_kg_cm = float(self.car_spec.get("servo_torque_kg_cm", 1.8))  # SG90 default = 1.8 kg*cm
        active_servo_kg_cm = 11.0 if self.upgrade_equipped else base_servo_kg_cm  # MG996R upgrade = 11.0 kg*cm
        torque_ok = active_servo_kg_cm >= required_torque_kg_cm

        return {
            "com_offset_m": [round(com_offset_x, 3), 0.0, 0.18],
            "wheel_footprint_m": [wheelbase_half_x, track_half_y],
            "com_stable": com_stable,
            "required_torque_kg_cm": required_torque_kg_cm,
            "rated_torque_kg_cm": active_servo_kg_cm,
            "torque_valid": torque_ok,
            "upgrade_equipped": self.upgrade_equipped,
            "motor_torque_nm": float(self.car_spec.get("motor_torque_nm", 3.5)),
            "wheel_friction": float(self.car_spec.get("wheel_friction", 1.2)),
        }

    def step(
        self,
        throttle: float,
        steering: float,
        equip_upgrade: Optional[bool] = None,
        substeps: int = 4,
    ) -> Dict[str, Any]:
        """Step the MuJoCo car simulation and return body transforms + physics verification gate status."""
        if equip_upgrade is not None and equip_upgrade != self.upgrade_equipped:
            self.upgrade_equipped = equip_upgrade
            if self.upgrade_equipped:
                self.car_spec["motor_torque_nm"] = 6.5
                self.car_spec["wheel_friction"] = 1.65
                self.car_spec["servo_torque_kg_cm"] = 11.0

        if self._mj_model is not None and self._mj_data is not None:
            try:
                import mujoco

                # Apply controls: 4 wheel motors + 2 front steering servos
                self._mj_data.ctrl[0] = throttle
                self._mj_data.ctrl[1] = throttle
                self._mj_data.ctrl[2] = throttle
                self._mj_data.ctrl[3] = throttle
                self._mj_data.ctrl[4] = steering * 0.55
                self._mj_data.ctrl[5] = steering * 0.55

                for _ in range(substeps):
                    mujoco.mj_step(self._mj_model, self._mj_data)

                chassis_id = mujoco.mj_name2id(self._mj_model, mujoco.mjtObj.mjOBJ_BODY, "car_chassis")
                pos = self._mj_data.xpos[chassis_id]
                quat = self._mj_data.xquat[chassis_id]  # [w, x, y, z] in MuJoCo (Z-up)

                # Map MuJoCo (X, Y, Z-up) -> Bevy (X, Y-up, Z)
                bevy_x = float(pos[0])
                bevy_y = float(pos[2])
                bevy_z = float(pos[1])
                vel = self._mj_data.cvel[chassis_id]
                speed_mps = float(math.hypot(vel[3], vel[4]))

                return {
                    "engine": "mujoco_native",
                    "chassis": {
                        "x": round(bevy_x, 4),
                        "y": round(bevy_y, 4),
                        "z": round(bevy_z, 4),
                        "quat_wxyz": [round(float(q), 4) for q in quat],
                        "speed_mps": round(speed_mps, 2),
                    },
                    "sanity_check": self.compute_physics_sanity_check(),
                }
            except Exception as exc:
                print(f"[MuJoCo] Step fallback ({exc})")

        # Physics-aware Ackermann + friction analytical step (if MuJoCo C-lib not active)
        dt = 0.02
        friction = float(self.car_spec.get("wheel_friction", 1.2))
        torque = float(self.car_spec.get("motor_torque_nm", 3.5))
        mass = float(self.car_spec.get("chassis_mass_kg", 4.5))

        accel = (throttle * torque * friction * 2.2) / mass
        self.speed = (self.speed + accel * dt) * 0.92
        if abs(self.speed) > 0.02:
            self.yaw += steering * 1.9 * dt * (1.0 if self.speed >= 0 else -1.0)

        nx = self.pos_x + math.cos(self.yaw) * self.speed * dt
        nz = self.pos_y + math.sin(self.yaw) * self.speed * dt

        # Check solid obstacles in level_entities
        blocked = False
        for ent in self.level_entities:
            if ent.get("solid", False):
                ex = float(ent.get("x", 999.0))
                ez = float(ent.get("z", 999.0))
                scale = float(ent.get("scale", 1.5))
                if abs(nx - ex) < (0.7 + 0.55 * scale) and abs(nz - ez) < (0.7 + 0.55 * scale):
                    blocked = True
                    self.speed = -self.speed * 0.25
                    break

        if not blocked:
            self.pos_x = nx
            self.pos_y = nz

        self.wheel_spin += self.speed * dt * 5.0
        half_yaw = -self.yaw * 0.5
        qw = math.cos(half_yaw)
        qy = math.sin(half_yaw)

        return {
            "engine": "mujoco_kinematic_solver",
            "chassis": {
                "x": round(self.pos_x, 4),
                "y": round(self.pos_z, 4),
                "z": round(self.pos_y, 4),
                "quat_wxyz": [round(qw, 4), 0.0, round(qy, 4), 0.0],
                "speed_mps": round(abs(self.speed), 2),
            },
            "sanity_check": self.compute_physics_sanity_check(),
        }
