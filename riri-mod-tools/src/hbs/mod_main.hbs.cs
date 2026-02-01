// This file was automatically generated.
// DO NOT EDIT THIS. It will get overwritten if you rebuild {{mod_name}}!
#nullable enable
using {{mod_id}}.Configuration;
using {{mod_id}}.Template;
using Reloaded.Hooks.ReloadedII.Interfaces;
using Reloaded.Memory.SigScan.ReloadedII.Interfaces;
using Reloaded.Mod.Interfaces;
using Reloaded.Mod.Interfaces.Internal;
{{#if uses_shared_scans}}
using SharedScans.Interfaces;
{{/if}}
using System.Drawing;
{{#if csharp_function_invoke}}
using System.Runtime.InteropServices;
{{/if}}
namespace {{mod_id}}
{
    {{#if exports_interfaces}}
    public partial class Mod : ModBase, IExports
    {{else}}
    public partial class Mod : ModBase
    {{/if}}
    {	
	    private static IModLoader? _modLoader;
        private static IReloadedHooks? _hooks;
        private static ILogger? _logger;
        private static IMod? _owner;
        private Config _configuration;
        private static IModConfig? _modConfig;
	    {{#if uses_shared_scans}}
	    private static ISharedScans? _sharedScans;
	    {{/if}}
	    private readonly IStartupScanner _startupScanner;
	    private static nint _baseAddress;
		private static Mod? _instance;

	    public void SigScan(string pattern, string name, Action<nuint> hookerCb)
	    {
	        {{#if cached_signatures}}
            if (CachedSignature != null)
            {
                using var Reader = new BinaryReader(new MemoryStream(CachedSignature));
                Reader.BaseStream.Seek(0xc, SeekOrigin.Begin); // ModCount
                var ModCount = Reader.ReadUInt32();
                Reader.BaseStream.Seek(0x10 * ModCount, SeekOrigin.Current);
                var SigCount = Reader.ReadUInt64();
                for (ulong i = 0; i < SigCount; i++)
                {
                    var (Hash, Offset) = (Reader.ReadUInt64(), Reader.ReadUInt64());
                    if (pattern.ToXxh3() == Hash)
                    {
                        CacheSigCallbacks += () => hookerCb((nuint)Offset);
                        return;
                    }
                }
                _logger!.WriteLineAsync($"Couldn't find location for {name}, stuff will break :(", Color.Red);
            }
            else
            {
                _startupScanner.AddMainModuleScan(pattern, result =>
                {
                    if (!result.Found)
                    {
                        _logger!.WriteLineAsync($"Couldn't find location for {name}, stuff will break :(", Color.Red);
                        return;
                    }
                    RegenSigs.Add(new RegenSigEntry(pattern.ToXxh3(), (ulong)result.Offset));
                    hookerCb((nuint)result.Offset);
                });
            }
	        {{else}}
            _startupScanner.AddMainModuleScan(pattern, result =>
            {
                if (!result.Found)
                   {
                       _logger!.WriteLineAsync($"Couldn't find location for {name}, stuff will break :(", Color.Red);
                       return;
                   }
                   hookerCb((nuint)result.Offset);
            });
	        {{/if}}
	    }

	    private IControllerType GetDependency<IControllerType>(string modName) where IControllerType : class
        {
            var controller = _modLoader!.GetController<IControllerType>();
            if (controller == null || !controller.TryGetTarget(out var target))
                throw new Exception($"{{mod_name}}: Could not get controller for \"{modName}\". This depedency is likely missing.");
            return target;
        }

	    public Mod(ModContext context)
	    {
	        _modLoader = context.ModLoader;
            _hooks = context.Hooks;
            _logger = context.Logger;
            _owner = context.Owner;
            _configuration = context.Configuration;
            _modConfig = context.ModConfig;
	    	var process = System.Diagnostics.Process.GetCurrentProcess();
	    	if (process == null || process.MainModule == null) throw new Exception("{{mod_name}}: Process is null");
	    	_baseAddress = process.MainModule.BaseAddress;

	        // Register mod interfaces
	        if (_hooks == null) throw new Exception("{{mod_name}}: Could not get controller for Reloaded hooks");
	        _startupScanner = GetDependency<IStartupScanner>("Reloaded Startup Scanner");
	        {{#if uses_shared_scans}}
	        _sharedScans = GetDependency<ISharedScans>("Shared Scans");
	        {{/if}}
	        {{utility_namespace}}.set_current_process();
	        {{#if cached_signatures}}
	        CheckSignatureCache();
	        {{/if}}
	        RegisterLogger();
			RegisterModLoaderAPI();
	        // Register hooks
	        {{#each register_hook_fn}}
	        {{this}}();
	        {{/each}}
			_instance = this;

			_modLoader!.OnModLoaderInitialized += OnLoaderInit;
			_modLoader!.ModLoading += OnModLoading;
	    }

		private void OnLoaderInit() 
		{
			{{#each loader_init_fn}}
	        {{this}}();
	        {{/each}}

	        {{#if cached_signatures}}
	        FinishSignatureCache();
	        {{/if}}

			_modLoader!.OnModLoaderInitialized -= OnLoaderInit;
			_modLoader!.ModLoading -= OnModLoading;
		}

		private void OnModLoading(IModV1 mod, IModConfigV1 conf)
		{
		    {{#if csharp_function_invoke}}
		    {{utility_namespace}}.on_mod_loading(GCHandle.ToIntPtr(GCHandle.Alloc(conf)));
		    {{/if}}
		}

	    public override void ConfigurationUpdated(Config configuration)
        {
            _configuration = configuration;
            _logger!.WriteLine($"{{mod_name}}: Config updated, applying...");
        }
#pragma warning disable CS8618
        public Mod() { }
#pragma warning restore CS8618
    }
}
