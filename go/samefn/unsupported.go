//go:build cgo && !((darwin && arm64) || (linux && (amd64 || arm64)))

package samefn

// No prebuilt samefn_<goos>_<goarch>.syso for this platform: fail at compile time with this
// name rather than with undefined symbols at link time.
var _ = noPrebuiltSamefnObjectForThisPlatform
