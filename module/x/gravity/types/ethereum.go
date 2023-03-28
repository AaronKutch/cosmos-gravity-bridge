package types

import (
	"bytes"
	"fmt"
	"regexp"
	"strings"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	// GravityDenomPrefix indicates the prefix for all assests minted by this module
	GravityDenomPrefix = "neon"

	// GravityDenomSeparator is the separator for gravity denoms
	GravityDenomSeparator = ""

	// ETHContractAddressLen is the length of contract address strings
	ETHContractAddressLen = 42

	// GravityDenomLen is the length of the denoms generated by the gravity module
	GravityDenomLen = len(GravityDenomPrefix) + len(GravityDenomSeparator) + ETHContractAddressLen

	// ZeroAddress is an EthAddress containing the zero ethereum address
	ZeroAddressString = "0x0000000000000000000000000000000000000000"
)

// Regular EthAddress
type EthAddress struct {
	address string
}

// Returns the contained address as a string
func (ea EthAddress) GetAddress() string {
	return ea.address
}

// Sets the contained address, performing validation before updating the value
func (ea EthAddress) SetAddress(address string) error {
	if err := ValidateEthAddress(address); err != nil {
		return err
	}
	ea.address = address
	return nil
}

// Creates a new EthAddress from a string, performing validation and returning any validation errors
func NewEthAddress(address string) (*EthAddress, error) {
	if err := ValidateEthAddress(address); err != nil {
		return nil, sdkerrors.Wrap(err, "invalid input address")
	}
	addr := EthAddress{address}
	return &addr, nil
}

// Returns a new EthAddress with 0x0000000000000000000000000000000000000000 as the wrapped address
func ZeroAddress() EthAddress {
	return EthAddress{ZeroAddressString}
}

// Validates the input string as an Ethereum Address
// Addresses must not be empty, have 42 character length, start with 0x and have 40 remaining characters in [0-9a-fA-F]
func ValidateEthAddress(address string) error {
	if address == "" {
		return fmt.Errorf("empty")
	}
	if len(address) != ETHContractAddressLen {
		return fmt.Errorf("address(%s) of the wrong length exp(%d) actual(%d)", address, ETHContractAddressLen, len(address))
	}
	if !regexp.MustCompile("^0x[0-9a-fA-F]{40}$").MatchString(address) {
		return fmt.Errorf("address(%s) doesn't pass regex", address)
	}

	return nil
}

// Performs validation on the wrapped string
func (ea EthAddress) ValidateBasic() error {
	return ValidateEthAddress(ea.address)
}

// EthAddrLessThan migrates the Ethereum address less than function
func EthAddrLessThan(e EthAddress, o EthAddress) bool {
	return bytes.Compare([]byte(e.GetAddress())[:], []byte(o.GetAddress())[:]) == -1
}

/////////////////////////
//     ERC20Token      //
/////////////////////////

// NewERC20Token returns a new instance of an ERC20
func NewERC20Token(amount uint64, contract string) ERC20Token {
	return ERC20Token{Amount: sdk.NewIntFromUint64(amount), Contract: contract}
}

// NewSDKIntERC20Token returns a new instance of an ERC20, accepting a sdk.Int
func NewSDKIntERC20Token(amount sdk.Int, contract string) ERC20Token {
	return ERC20Token{Amount: amount, Contract: contract}
}

// ToInternal converts an ERC20Token to the internal type InternalERC20Token
func (e ERC20Token) ToInternal() (*InternalERC20Token, error) {
	return NewInternalERC20Token(e.Amount, e.Contract)
}

// InternalERC20Token contains validated fields, used for all internal computation
type InternalERC20Token struct {
	Amount   sdk.Int
	Contract EthAddress
}

// NewInternalERC20Token creates an InternalERC20Token, performing validation and returning any errors
func NewInternalERC20Token(amount sdk.Int, contract string) (*InternalERC20Token, error) {
	ethAddress, err := NewEthAddress(contract)
	if err != nil { // ethAddress could be nil, must return here
		return nil, sdkerrors.Wrap(err, "invalid contract")
	}
	ret := &InternalERC20Token{
		Amount:   amount,
		Contract: *ethAddress,
	}
	if err := ret.ValidateBasic(); err != nil {
		return nil, err
	}

	return ret, nil
}

// ValidateBasic performs validation on all fields of an InternalERC20Token
func (i *InternalERC20Token) ValidateBasic() error {
	if i.Amount.IsNegative() {
		return sdkerrors.Wrap(sdkerrors.ErrInvalidCoins, "coins must not be negative")
	}
	err := i.Contract.ValidateBasic()
	if err != nil {
		return sdkerrors.Wrap(err, "invalid contract")
	}
	return nil
}

// ToExternal converts an InternalERC20Token to the external type ERC20Token
func (i *InternalERC20Token) ToExternal() ERC20Token {
	return ERC20Token{
		Amount:   i.Amount,
		Contract: i.Contract.GetAddress(),
	}
}

// GravityCoin returns the gravity representation of the ERC20
func (i *InternalERC20Token) GravityCoin() sdk.Coin {
	return sdk.NewCoin(GravityDenom(i.Contract), i.Amount)
}

// GravityDenom converts an EthAddress to a gravity cosmos denom
func GravityDenom(tokenContract EthAddress) string {
	return fmt.Sprintf("%s%s%s", GravityDenomPrefix, GravityDenomSeparator, tokenContract.GetAddress())
}

// ValidateBasic permforms stateless validation
func (e *ERC20Token) ValidateBasic() error {
	if err := ValidateEthAddress(e.Contract); err != nil {
		return sdkerrors.Wrap(err, "ethereum address")
	}
	// TODO: Validate all the things
	return nil
}

// Add adds one ERC20 to another
// TODO: make this return errors instead
func (i *InternalERC20Token) Add(o *InternalERC20Token) (*InternalERC20Token, error) {
	if i.Contract.GetAddress() != o.Contract.GetAddress() {
		return nil, sdkerrors.Wrap(ErrMismatched, "cannot add two different tokens")
	}
	sum := i.Amount.Add(o.Amount) // validation happens in NewInternalERC20Token()
	return NewInternalERC20Token(sum, i.Contract.GetAddress())
}

// GravityDenomToERC20 converts a gravity cosmos denom to an EthAddress
func GravityDenomToERC20(denom string) (*EthAddress, error) {
	fullPrefix := GravityDenomPrefix + GravityDenomSeparator
	if !strings.HasPrefix(denom, fullPrefix) {
		return nil, fmt.Errorf("denom prefix(%s) not equal to expected(%s)", denom, fullPrefix)
	}
	contract := strings.TrimPrefix(denom, fullPrefix)
	ethAddr, err := NewEthAddress(contract)
	switch {
	case err != nil:
		return nil, fmt.Errorf("error(%s) validating ethereum contract address", err)
	case len(denom) != GravityDenomLen:
		return nil, fmt.Errorf("len(denom)(%d) not equal to GravityDenomLen(%d)", len(denom), GravityDenomLen)
	default:
		return ethAddr, nil
	}
}
