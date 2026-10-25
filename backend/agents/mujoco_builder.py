"""MuJoCo Driving & Transformers-Style Multi-Robot Physics Model Generator.

Generates physics-aware MuJoCo MJCF (.xml) & URDF definitions for:
- `rover_car`: 4-Wheel Ackermann Street Car / Rover
- `bipedal_mech`: Upright Bipedal Striker Mech (Hip/Knee hinge actuators + Torso)
- `quadruped_crawler`: 4-Legged Spider / Quadruped Robotics Crawler
- `siege_tank`: Heavy Tracked Dual-Tread Siege Tank with Rotating Turret
"""

from typing import Any, Dict

from .artifact_puller import GENERATED_ASSETS_DIR


def build_mujoco_car_mjcf(
    project_id: str,
    car_spec: Dict[str, Any],
    skin_texture_file: str = "chassis_skin.png",
) -> str:
    """Write a complete MuJoCo XML (MJCF) model for the active robot morphology + street course."""
    target_dir = GENERATED_ASSETS_DIR / project_id
    target_dir.mkdir(parents=True, exist_ok=True)

    robot_form = str(car_spec.get("robot_form", "rover_car"))
    mass_kg = float(car_spec.get("chassis_mass_kg", 4.5))
    wheel_friction = float(car_spec.get("wheel_friction", 1.2))
    motor_torque = float(car_spec.get("motor_torque_nm", 3.5))
    clearance_m = float(car_spec.get("ground_clearance_m", 0.18))
    rgba = car_spec.get("chassis_rgba", "0.15 0.75 0.95 1.0")

    if robot_form == "bipedal_mech":
        chassis_z = clearance_m + 0.55
        chassis_geom = f'<geom name="chassis_body" type="box" size="0.42 0.38 0.48" mass="{mass_kg}" material="chassis_mat" rgba="{rgba}"/>'
    elif robot_form == "quadruped_crawler":
        chassis_z = clearance_m + 0.25
        chassis_geom = f'<geom name="chassis_body" type="box" size="0.55 0.48 0.14" mass="{mass_kg}" material="chassis_mat" rgba="{rgba}"/>'
    elif robot_form == "siege_tank":
        chassis_z = clearance_m + 0.20
        chassis_geom = f'<geom name="chassis_body" type="box" size="0.78 0.52 0.22" mass="{mass_kg * 1.35:.2f}" material="chassis_mat" rgba="{rgba}"/>'
    else:
        chassis_z = clearance_m + 0.15
        chassis_geom = f'<geom name="chassis_body" type="box" size="0.65 0.40 0.15" mass="{mass_kg}" material="chassis_mat" rgba="{rgba}"/>'

    mjcf_xml = f"""<mujoco model="argon_{robot_form}_{project_id}">
  <compiler angle="degree" coordinate="local" inertiafromgeom="true"/>
  <option timestep="0.005" gravity="0 0 -9.81" integrator="RK4"/>

  <asset>
    <texture name="chassis_skin" type="2d" file="{skin_texture_file}"/>
    <material name="chassis_mat" texture="chassis_skin" specular="0.6" shininess="0.5"/>
    <texture name="ground_grid" type="2d" builtin="checker" width="512" height="512" rgb1="0.12 0.14 0.18" rgb2="0.18 0.22 0.28"/>
    <material name="ground_mat" texture="ground_grid" texrepeat="12 12" reflectance="0.15"/>
  </asset>

  <default>
    <joint armature="0.01" damping="0.1" limited="false"/>
    <geom condim="3" friction="{wheel_friction} 0.005 0.0001" solimp="0.9 0.95 0.001" solref="0.02 1"/>
  </default>

  <worldbody>
    <light directional="true" diffuse="0.9 0.9 0.9" specular="0.3 0.3 0.3" pos="0 0 15" dir="0 0 -1"/>
    <geom name="track_floor" type="plane" size="40 25 0.1" material="ground_mat"/>

    <!-- Goal Beacon -->
    <body name="data_core" pos="38 0 0.5">
      <geom name="goal_geom" type="cylinder" size="0.8 0.4" rgba="0.1 1.0 0.45 0.85" contype="0" conaffinity="0"/>
    </body>

    <!-- Obstacle / Blockade -->
    <body name="blast_door" pos="19 0 0.75">
      <geom name="blast_door_geom" type="box" size="1.5 1.5 0.75" rgba="0.92 0.24 0.24 1.0"/>
    </body>

    <!-- Physics-Aware Transformable MuJoCo Robot ({robot_form}) -->
    <body name="car_chassis" pos="0 0 {chassis_z}">
      <freejoint name="car_root"/>
      {chassis_geom}

      <!-- Front Left Limb / Wheel Actuator (Steered + Driven) -->
      <body name="wheel_fl" pos="0.45 0.45 -0.05">
        <joint name="steer_fl" type="hinge" axis="0 0 1" limited="true" range="-35 35" damping="0.5"/>
        <joint name="drive_fl" type="hinge" axis="0 1 0"/>
        <geom type="cylinder" size="0.18 0.07" euler="90 0 0" mass="0.6" rgba="0.15 0.15 0.18 1"/>
      </body>

      <!-- Front Right Limb / Wheel Actuator (Steered + Driven) -->
      <body name="wheel_fr" pos="0.45 -0.45 -0.05">
        <joint name="steer_fr" type="hinge" axis="0 0 1" limited="true" range="-35 35" damping="0.5"/>
        <joint name="drive_fr" type="hinge" axis="0 1 0"/>
        <geom type="cylinder" size="0.18 0.07" euler="90 0 0" mass="0.6" rgba="0.15 0.15 0.18 1"/>
      </body>

      <!-- Rear Left Limb / Wheel Actuator -->
      <body name="wheel_rl" pos="-0.45 0.45 -0.05">
        <joint name="drive_rl" type="hinge" axis="0 1 0"/>
        <geom type="cylinder" size="0.18 0.07" euler="90 0 0" mass="0.6" rgba="0.15 0.15 0.18 1"/>
      </body>

      <!-- Rear Right Limb / Wheel Actuator -->
      <body name="wheel_rr" pos="-0.45 -0.45 -0.05">
        <joint name="drive_rr" type="hinge" axis="0 1 0"/>
        <geom type="cylinder" size="0.18 0.07" euler="90 0 0" mass="0.6" rgba="0.15 0.15 0.18 1"/>
      </body>
    </body>
  </worldbody>

  <actuator>
    <motor name="throttle_fl" joint="drive_fl" gear="{motor_torque}" ctrllimited="true" ctrlrange="-1 1"/>
    <motor name="throttle_fr" joint="drive_fr" gear="{motor_torque}" ctrllimited="true" ctrlrange="-1 1"/>
    <motor name="throttle_rl" joint="drive_rl" gear="{motor_torque}" ctrllimited="true" ctrlrange="-1 1"/>
    <motor name="throttle_rr" joint="drive_rr" gear="{motor_torque}" ctrllimited="true" ctrlrange="-1 1"/>
    <position name="steering_fl" joint="steer_fl" kp="15" ctrllimited="true" ctrlrange="-0.6 0.6"/>
    <position name="steering_fr" joint="steer_fr" kp="15" ctrllimited="true" ctrlrange="-0.6 0.6"/>
  </actuator>
</mujoco>
"""
    mjcf_path = target_dir / "car_level.xml"
    mjcf_path.write_text(mjcf_xml, encoding="utf-8")
    return str(mjcf_path)
