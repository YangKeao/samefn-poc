// Package obj holds samefn's prebuilt code, one object per platform
// (samefn_<goos>_<goarch>.syso, built by scripts/build-go-object.sh). Go links a package's
// .syso files whether or not cgo is enabled, so only the cgo side of package samefn imports
// this package: a build without cgo never links the objects.
package obj
