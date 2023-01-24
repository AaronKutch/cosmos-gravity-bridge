package keeper

import (
	"bytes"
	"fmt"
	"sort"
	"testing"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	gethcommon "github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/onomyprotocol/arc/module/x/gravity/types"
)

//nolint: exhaustivestruct
func TestPrefixRange(t *testing.T) {
	cases := map[string]struct {
		src      []byte
		expStart []byte
		expEnd   []byte
		expPanic bool
	}{
		"normal":              {src: []byte{1, 3, 4}, expStart: []byte{1, 3, 4}, expEnd: []byte{1, 3, 5}},
		"normal short":        {src: []byte{79}, expStart: []byte{79}, expEnd: []byte{80}},
		"empty case":          {src: []byte{}},
		"roll-over example 1": {src: []byte{17, 28, 255}, expStart: []byte{17, 28, 255}, expEnd: []byte{17, 29, 0}},
		"roll-over example 2": {src: []byte{15, 42, 255, 255},
			expStart: []byte{15, 42, 255, 255}, expEnd: []byte{15, 43, 0, 0}},
		"pathological roll-over": {src: []byte{255, 255, 255, 255}, expStart: []byte{255, 255, 255, 255}},
		"nil prohibited":         {expPanic: true},
	}

	for testName, tc := range cases {
		tc := tc
		t.Run(testName, func(t *testing.T) {
			if tc.expPanic {
				require.Panics(t, func() {
					prefixRange(tc.src)
				})
				return
			}
			start, end := prefixRange(tc.src)
			assert.Equal(t, tc.expStart, start)
			assert.Equal(t, tc.expEnd, end)
		})
	}
}

// Test that valset creation produces the expected normalized power values
//nolint: exhaustivestruct
func TestCurrentValsetNormalization(t *testing.T) {
	// Setup the overflow test
	maxPower64 := make([]uint64, 64)             // users with max power (approx 2^63)
	expPower64 := make([]uint64, 64)             // expected scaled powers
	ethAddrs64 := make([]gethcommon.Address, 64) // need 64 eth addresses for this test
	for i := 0; i < 64; i++ {
		maxPower64[i] = uint64(9223372036854775807)
		expPower64[i] = 67108864 // 2^32 split amongst 64 validators
		ethAddrs64[i] = gethcommon.BytesToAddress(bytes.Repeat([]byte{byte(i)}, 20))
	}

	// any lower than this and a validator won't be created
	const minStake = 1000000

	specs := map[string]struct {
		srcPowers []uint64
		expPowers []uint64
	}{
		"one": {
			srcPowers: []uint64{minStake},
			expPowers: []uint64{4294967296},
		},
		"two": {
			srcPowers: []uint64{minStake * 99, minStake * 1},
			expPowers: []uint64{4252017623, 42949672},
		},
		"four equal": {
			srcPowers: []uint64{minStake, minStake, minStake, minStake},
			expPowers: []uint64{1073741824, 1073741824, 1073741824, 1073741824},
		},
		"four equal max power": {
			srcPowers: []uint64{4294967296, 4294967296, 4294967296, 4294967296},
			expPowers: []uint64{1073741824, 1073741824, 1073741824, 1073741824},
		},
		"overflow": {
			srcPowers: maxPower64,
			expPowers: expPower64,
		},
	}
	for msg, spec := range specs {
		spec := spec
		t.Run(msg, func(t *testing.T) {
			input, ctx := SetupTestChain(t, spec.srcPowers, true)
			r, err := input.GravityKeeper.GetCurrentValset(ctx)
			require.NoError(t, err)
			rMembers, err := types.BridgeValidators(r.Members).ToInternal()
			require.NoError(t, err)
			assert.Equal(t, spec.expPowers, rMembers.GetPowers())
		})
	}
}

//nolint: exhaustivestruct
func TestAttestationIterator(t *testing.T) {
	input := CreateTestEnv(t)
	ctx := input.Context
	// add some attestations to the store

	att1 := &types.Attestation{
		Observed: true,
		Votes:    []string{},
	}
	dep1 := &types.MsgSendToCosmosClaim{
		EventNonce:     1,
		TokenContract:  TokenContractAddrs[0],
		Amount:         sdk.NewInt(100),
		EthereumSender: EthAddrs[0].String(),
		CosmosReceiver: AccAddrs[0].String(),
		Orchestrator:   AccAddrs[0].String(),
	}
	att2 := &types.Attestation{
		Observed: true,
		Votes:    []string{},
	}
	dep2 := &types.MsgSendToCosmosClaim{
		EventNonce:     2,
		TokenContract:  TokenContractAddrs[0],
		Amount:         sdk.NewInt(100),
		EthereumSender: EthAddrs[0].String(),
		CosmosReceiver: AccAddrs[0].String(),
		Orchestrator:   AccAddrs[0].String(),
	}
	hash1, err := dep1.ClaimHash()
	require.NoError(t, err)
	hash2, err := dep2.ClaimHash()
	require.NoError(t, err)

	input.GravityKeeper.SetAttestation(ctx, dep1.EventNonce, hash1, att1)
	input.GravityKeeper.SetAttestation(ctx, dep2.EventNonce, hash2, att2)

	atts := []types.Attestation{}
	input.GravityKeeper.IterateAttestaions(ctx, func(_ []byte, att types.Attestation) bool {
		atts = append(atts, att)
		return false
	})

	require.Len(t, atts, 2)
}

//nolint: exhaustivestruct
func TestDelegateKeys(t *testing.T) {
	input := CreateTestEnv(t)
	ctx := input.Context
	k := input.GravityKeeper
	length := 4
	tmp_ethStrings := make([]string, length)
	tmp_ethAddrs := make([]types.EthAddress, length)
	tmp_valAddrs := make([]sdk.ValAddress, length)
	tmp_orchAddrs := make([]sdk.AccAddress, length)
	for i := 0; i < length; i++ {
		// we need the strings for both sorting and an assertion below
		tmp_ethStrings[i] = fmt.Sprintf("0x%s", secp256k1.GenPrivKey().PubKey().Address().String())
		tmp_valAddrs[i] = RandomValAddress()
		tmp_orchAddrs[i] = RandomAccAddress()
	}
	sort.Strings(tmp_ethStrings)
	for i := 0; i < length; i++ {
		tmp, err := types.NewEthAddress(tmp_ethStrings[i])
		require.NoError(t, err)
		tmp_ethAddrs[i] = *tmp
	}
	var (
		ethStrings = tmp_ethStrings
		ethAddrs   = tmp_ethAddrs
		valAddrs   = tmp_valAddrs
		orchAddrs  = tmp_orchAddrs
	)

	for i := range ethAddrs {
		// set the orchestrator address
		k.SetOrchestratorValidator(ctx, valAddrs[i], orchAddrs[i])
		// set the ethereum address
		k.SetEthAddressForValidator(ctx, valAddrs[i], ethAddrs[i])
	}

	addresses := k.GetDelegateKeys(ctx)
	for i := range addresses {
		res := addresses[i]
		assert.Equal(t, valAddrs[i].String(), res.Validator)
		assert.Equal(t, orchAddrs[i].String(), res.Orchestrator)
		assert.Equal(t, ethStrings[i], res.EthAddress)
	}
}

//nolint: exhaustivestruct
func TestLastSlashedValsetNonce(t *testing.T) {
	input, ctx := SetupFiveValChain(t)
	k := input.GravityKeeper

	vs, err := k.GetCurrentValset(ctx)
	require.NoError(t, err)

	i := 1
	for ; i < 10; i++ {
		vs.Height = uint64(i)
		vs.Nonce = uint64(i)
		k.StoreValset(ctx, vs)
		k.SetLatestValsetNonce(ctx, vs.Nonce)
	}

	latestValsetNonce := k.GetLatestValsetNonce(ctx)
	assert.Equal(t, latestValsetNonce, uint64(i-1))

	//  lastSlashedValsetNonce should be zero initially.
	lastSlashedValsetNonce := k.GetLastSlashedValsetNonce(ctx)
	assert.Equal(t, lastSlashedValsetNonce, uint64(0))
	unslashedValsets := k.GetUnSlashedValsets(ctx, uint64(12))
	assert.Equal(t, len(unslashedValsets), 9)

	// check if last Slashed Valset nonce is set properly or not
	k.SetLastSlashedValsetNonce(ctx, uint64(3))
	lastSlashedValsetNonce = k.GetLastSlashedValsetNonce(ctx)
	assert.Equal(t, lastSlashedValsetNonce, uint64(3))

	lastSlashedValset := k.GetValset(ctx, lastSlashedValsetNonce)

	// when valset height + signedValsetsWindow > current block height, len(unslashedValsets) should be zero
	unslashedValsets = k.GetUnSlashedValsets(ctx, uint64(ctx.BlockHeight()))
	assert.Equal(t, len(unslashedValsets), 0)

	// when lastSlashedValset height + signedValsetsWindow == BlockHeight, len(unslashedValsets) should be zero
	heightDiff := uint64(ctx.BlockHeight()) - lastSlashedValset.Height
	unslashedValsets = k.GetUnSlashedValsets(ctx, heightDiff)
	assert.Equal(t, len(unslashedValsets), 0)

	// when signedValsetsWindow is between lastSlashedValset height and latest valset's height
	unslashedValsets = k.GetUnSlashedValsets(ctx, heightDiff-2)
	assert.Equal(t, len(unslashedValsets), 2)

	// when signedValsetsWindow > latest valset's height
	unslashedValsets = k.GetUnSlashedValsets(ctx, heightDiff-6)
	assert.Equal(t, len(unslashedValsets), 6)
	fmt.Println("unslashedValsetsRange", unslashedValsets)
}
