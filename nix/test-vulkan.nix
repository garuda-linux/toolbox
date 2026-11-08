# Mesa's lavapipe, the CPU Vulkan driver the tests render on (scripts/test-vulkan.sh).
{ mesa, stdenv }:
"${mesa}/share/vulkan/icd.d/lvp_icd.${stdenv.hostPlatform.parsed.cpu.name}.json"
