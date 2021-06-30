// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;

import "./BeefyLightClient.sol";
import "./utils/MerkleProof.sol";

library ParachainLightClient {
    struct OwnParachainHead {
        bytes32 parentHash;
        uint32 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
        bytes32 commitment; // TODO check type and position of this element
    }

    struct OwnParachainHeadPartial {
        bytes32 parentHash;
        uint32 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
    }

    struct BeefyMMRLeafPartial {
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetId;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
    }

    function verifyCommitmentInParachain(
        bytes32 commitment,
        ParachainLightClient.OwnParachainHeadPartial
            calldata _ownParachainHeadPartial,
        bytes32[] calldata _parachainHeadsProof,
        BeefyMMRLeafPartial calldata _beefyMMRLeafPartial,
        uint256 _beefyMMRLeafIndex,
        uint256 _beefyMMRLeafCount,
        bytes32[] calldata _beefyMMRLeafProof
    ) internal {
        // Must verify the parachain id to ensure msg comes from our parachain
        // TODO

        // 2. Compute `ownParachainHead` by hashing the data of the `commitment` together with the contents of
        // `_ownParachainHeadPartial`
        bytes32 ownParachainHeadHash = encodeParachainHeadHash(
            _ownParachainHeadPartial,
            commitment
        );

        // 3. Compute `parachainHeadsRoot` by verifying the merkle proof using `ownParachainHeadHash` and
        // `_parachainHeadsProof`
        bytes32 parachainHeadsRoot = MerkleProof.computeMerkleLeafAtPosition(
            ownParachainHeadHash,
            pos,
            width,
            proof
        );

        // 4. Compute the `beefyMMRLeaf` using `parachainHeadsRoot` and `_beefyMMRLeafPartial`
        // TODO

        // 5. Verify inclusion of the beefy MMR leaf in the beefy MMR root using that `beefyMMRLeaf` as well as
        // `_beefyMMRLeafIndex`, `_beefyMMRLeafCount` and `_beefyMMRLeafProof`
        // TODO
        // require(
        //     beefyLightClient.verifyBeefyMerkleLeaf(
        //         beefyMMRLeaf,
        //         _beefyMMRLeafIndex,
        //         _beefyMMRLeafCount,
        //         _beefyMMRLeafProof
        //     ),
        //     "Invalid proof"
        // );
    }

    function encodeParachainHeadHash(
        ParachainLightClient.OwnParachainHeadPartial
            calldata _ownParachainHeadPartial,
        bytes32 commitment
    ) public pure returns (bytes memory) {
        return
            keccak256(
                abi.encode(
                    ParachainLightClient.OwnParachainHead(
                        _ownParachainHeadPartial.parentHash,
                        _ownParachainHeadPartial.number,
                        _ownParachainHeadPartial.stateRoot,
                        _ownParachainHeadPartial.extrinsicsRoot,
                        commitment
                    )
                )
            );
    }
}
