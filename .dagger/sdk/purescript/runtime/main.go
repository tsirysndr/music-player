// Dagger's custom-SDK adapter. User modules and their entrypoint are PureScript.
package main

import (
	"context"
	"fmt"
	"path"
	"path/filepath"
	"strings"

	"purescript-sdk/internal/dagger"
)

type PurescriptSdk struct {
	Source *dagger.Directory
}

// FindClientRoot locates the nearest PureScript package.
func (m *PurescriptSdk) FindClientRoot(ctx context.Context, ws *dagger.Workspace) (*string, error) {
	marker, err := ws.FindUp(ctx, "spago.yaml")
	if err != nil || marker == "" {
		return nil, err
	}
	root := strings.TrimPrefix(path.Dir(marker), "/")
	if root == "" {
		root = "."
	}
	return &root, nil
}

// GenerateScope writes configuration and dependency bindings before execution.
func (m *PurescriptSdk) GenerateScope(ctx context.Context, ws *dagger.Workspace, isModule bool, name string, clients []*dagger.ModuleSource) (*dagger.Workspace, error) {
	if !isModule {
		return nil, fmt.Errorf("standalone scopes are not supported yet; use a PureScript module scope")
	}
	cwd, err := ws.Cwd(ctx)
	if err != nil {
		return nil, err
	}
	ref, err := ws.SDK("purescript").Ref(ctx)
	if err != nil {
		return nil, err
	}
	if strings.HasPrefix(ref, ".") || strings.HasPrefix(ref, "/") || !strings.ContainsAny(strings.Split(ref, "/")[0], ".:@") {
		ref, err = filepath.Rel(path.Join("/", cwd), path.Join("/", ref))
		if err != nil {
			return nil, err
		}
	}
	exists, err := ws.Directory(".").Exists(ctx, "dagger-module.toml")
	if err != nil {
		return nil, err
	}
	if !exists {
		ws = ws.WithNewFile("dagger-module.toml", fmt.Sprintf("name = %q\nengineVersion = \"v1.0.0-beta.21\"\n[runtime]\nsource = %q\n", name, ref))
	}
	src := ws.ModuleSource("/" + strings.TrimPrefix(cwd, "/")).WithName(name).WithSDK(ref).WithDependencies(clients)
	root, err := src.SourceRootSubpath(ctx)
	if err != nil {
		return nil, err
	}
	return ws.WithDirectory(".", src.GeneratedContextDirectory().Directory(root)), nil
}

func New(ctx context.Context) (*PurescriptSdk, error) {
	subpath, err := dag.CurrentModule().SourceSubpath(ctx)
	if err != nil {
		return nil, err
	}
	return &PurescriptSdk{Source: dag.CurrentModule().ContextDirectory().Directory(path.Dir(subpath))}, nil
}

func (m *PurescriptSdk) Codegen(ctx context.Context, modSource *dagger.ModuleSource, introspectionJSON *dagger.File) (*dagger.GeneratedCode, error) {
	ctr, subpath, err := m.prepare(ctx, modSource, introspectionJSON)
	if err != nil {
		return nil, err
	}
	return dag.GeneratedCode(ctr.Directory("/src").
		WithoutDirectory(path.Join(subpath, "output")).
		WithoutDirectory(path.Join(subpath, ".spago")).
		WithoutDirectory(path.Join(subpath, "node_modules"))).
		WithVCSGeneratedPaths([]string{"dagger-sdk/**"}).
		WithVCSIgnoredPaths([]string{".spago", "output", "node_modules"}), nil
}

func (m *PurescriptSdk) ModuleRuntime(ctx context.Context, modSource *dagger.ModuleSource,
	// +optional
	introspectionJSON *dagger.File,
) (*dagger.Container, error) {
	ctr, subpath, err := m.prepare(ctx, modSource, introspectionJSON)
	if err != nil {
		return nil, err
	}
	return ctr.WithExec([]string{"spago", "build"}).
		WithEntrypoint([]string{"node", "--input-type=module", "-e", fmt.Sprintf("import(%q).then(m => m.main()).catch(e => { console.error(e); process.exitCode = 1; })", path.Join("/src", subpath, "output/Main/index.js"))}), nil
}

func (m *PurescriptSdk) prepare(ctx context.Context, modSource *dagger.ModuleSource, introspectionJSON *dagger.File) (*dagger.Container, string, error) {
	subpath, err := modSource.SourceSubpath(ctx)
	if err != nil {
		return nil, "", fmt.Errorf("module source path: %w", err)
	}
	ctr := dag.Container().From("node:22-bookworm-slim").
		WithExec([]string{"sh", "-c", "apt-get update && apt-get install -y --no-install-recommends ca-certificates git libtinfo6 && rm -rf /var/lib/apt/lists/*"}).
		WithExec([]string{"npm", "install", "--global", "purescript@0.15.16", "spago@1.0.5"})
	workdir := path.Join("/src", subpath)
	if introspectionJSON == nil {
		return ctr.WithDirectory("/src", modSource.ContextDirectory()).WithWorkdir(workdir).
			WithMountedCache("/root/.cache/spago-nodejs", dag.CacheVolume("purescript-spago-0.15.16")), subpath, nil
	}
	name, err := modSource.ModuleOriginalName(ctx)
	if err != nil {
		return nil, "", err
	}
	ctr = ctr.WithDirectory("/sdk", m.Source).
		WithFile("/schema.json", introspectionJSON).
		WithExec([]string{"node", "/sdk/scripts/generate.mjs", "/schema.json"})
	// Keep the generated package's source and manifest, without its workspace or tests.
	sdk := ctr.Directory("/sdk").WithoutDirectory("runtime").WithoutDirectory("examples").WithoutFile("schema/introspection.json").
		WithoutDirectory("test").WithoutDirectory(".spago").WithoutDirectory("output").
		WithoutDirectory("node_modules").WithoutFile("spago.lock")
	contextDir := modSource.ContextDirectory().
		WithoutDirectory(path.Join(subpath, "dagger-sdk")).
		WithoutDirectory(path.Join(subpath, ".spago")).
		WithoutDirectory(path.Join(subpath, "output")).
		WithoutDirectory(path.Join(subpath, "node_modules"))
	ctr = ctr.WithDirectory("/src", contextDir).
		WithDirectory(path.Join(workdir, "dagger-sdk"), sdk).
		WithWorkdir(workdir).
		WithExec([]string{"node", "/sdk/scripts/init-module.mjs", name}).
		WithMountedCache("/root/.cache/spago-nodejs", dag.CacheVolume("purescript-spago-0.15.16"))
	return ctr, subpath, nil
}
