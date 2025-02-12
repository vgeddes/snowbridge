//go:generate bash -c "jq .abi ../../ethereum/artifacts/contracts/BeefyClient.sol/BeefyClient.json | abigen --abi - --type BeefyClient --pkg beefyclient --out beefyclient/contract.go"
//go:generate bash -c "jq .abi ../../ethereum/artifacts/contracts/BasicInboundChannel.sol/BasicInboundChannel.json | abigen --abi - --type BasicInboundChannel --pkg basic --out basic/inbound.go"
//go:generate bash -c "jq .abi ../../ethereum/artifacts/contracts/BasicOutboundChannel.sol/BasicOutboundChannel.json | abigen --abi - --type BasicOutboundChannel --pkg basic --out basic/outbound.go"
//go:generate bash -c "jq .abi ../../ethereum/artifacts/contracts/IncentivizedInboundChannel.sol/IncentivizedInboundChannel.json | abigen --abi - --type IncentivizedInboundChannel --pkg incentivized --out incentivized/inbound.go"
//go:generate bash -c "jq .abi ../../ethereum/artifacts/contracts/IncentivizedOutboundChannel.sol/IncentivizedOutboundChannel.json | abigen --abi - --type IncentivizedOutboundChannel --pkg incentivized --out incentivized/outbound.go"

package contracts
