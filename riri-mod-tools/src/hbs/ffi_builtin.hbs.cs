// This file was automatically generated.
// DO NOT EDIT THIS. It will get overwritten if you rebuild {{mod_name}}!
// (btw, please keep this in sync with extern definitions in riri-mod-tools-rt! ^^;)

{{#if csharp_function_invoke}}
using Minerals.StringCases;
using Reloaded.Memory.Extensions;
using Reloaded.Mod.Interfaces;
{{/if}}
using Reloaded.Memory.Sigscan;
{{#if csharp_function_invoke}}
using System.Collections;
using System.Collections.Concurrent;
using System.Diagnostics.CodeAnalysis;
{{/if}}
using System.Drawing;
using System.IO.Hashing;
{{#if csharp_function_invoke}}
using System.Reflection;
{{/if}}
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;

namespace {{ffi_namespace}} 
{
    internal static unsafe partial class Utilities 
    {
		const string __DllName = "{{dll_name}}";

		// Sigscan Resolvers

		[DllImport(__DllName, EntryPoint = "get_address", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint get_address(nuint offset);

		[DllImport(__DllName, EntryPoint = "get_address_may_thunk", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint get_address_may_thunk(nuint offset);

		[DllImport(__DllName, EntryPoint = "get_indirect_address_short", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint get_indirect_address_short(nuint offset);

		[DllImport(__DllName, EntryPoint = "get_indirect_address_short2", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint get_indirect_address_short2(nuint offset);

		[DllImport(__DllName, EntryPoint = "get_indirect_address_long", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint get_indirect_address_long(nuint offset);

		[DllImport(__DllName, EntryPoint = "get_indirect_address_long4", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint get_indirect_address_long4(nuint offset);

		[DllImport(__DllName, EntryPoint = "set_current_process", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern void set_current_process();

		// Logger
		[DllImport(__DllName, EntryPoint = "set_reloaded_logger", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint set_reloaded_logger(delegate* unmanaged[Stdcall]<nint, nint, int, int, byte, void> offset);

		[DllImport(__DllName, EntryPoint = "set_reloaded_logger_newline", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint set_reloaded_logger_newline(delegate* unmanaged[Stdcall]<nint, nint, int, int, byte, void> offset);

		[DllImport(__DllName, EntryPoint = "set_logger_color", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern bool set_logger_color(nint colorName);

		// Logger callback

		[DllImport(__DllName, EntryPoint = "on_reloaded_logger", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
    	internal static extern void on_reloaded_logger(nint pString);

    	[DllImport(__DllName, EntryPoint = "on_reloaded_logger_newline", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
    	internal static extern void on_reloaded_logger_newline(nint pString);

		// Executable hash
		[DllImport(__DllName, EntryPoint = "get_executable_hash", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern ulong get_executable_hash();
		
		// Vtable
		[DllImport(__DllName, EntryPoint = "get_vtable_rtti", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint get_vtable_rtti(nuint name, uint offset);

		// Mod Loader actions (temporary)
		[DllImport(__DllName, EntryPoint = "set_get_directory_for_mod", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint set_get_directory_for_mod(delegate* unmanaged[Stdcall]<nint> offset);

        [DllImport(__DllName, EntryPoint = "set_get_config_directory", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
        internal static extern nuint set_get_config_directory(delegate* unmanaged[Stdcall]<nint> offset);

		[DllImport(__DllName, EntryPoint = "set_free_csharp_string", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern nuint set_free_csharp_string(delegate* unmanaged[Stdcall]<nint, void> offset);

		// Stack Trace builder
		// [DllImport(__DllName, EntryPoint = "set_get_stack_trace", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		// internal static extern void set_get_stack_trace(delegate* unmanaged[Stdcall]<nint> offset);

		// Arbitrary Signature Scanning
		[DllImport(__DllName, EntryPoint = "set_find_pattern", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern void set_find_pattern(delegate* unmanaged[Stdcall]<nint, nint, int, nint> offset);

        {{#if csharp_function_invoke}}
        // Arbitrary C# function invocation
		[DllImport(__DllName, EntryPoint = "set_push_parameter", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern void set_push_parameter(delegate* unmanaged[Stdcall]<ulong, ulong, nint, void> offset);

		[DllImport(__DllName, EntryPoint = "set_call_function", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
		internal static extern void set_call_function(delegate* unmanaged[Stdcall]<ulong, ulong, nint, nint> offset);

		[DllImport(__DllName, EntryPoint = "set_free_object", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
    	internal static extern void set_free_object(delegate* unmanaged[Stdcall]<nint, void> offset);

    	[DllImport(__DllName, EntryPoint = "set_object_as_u8", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
    	internal static extern void set_object_as_u8(delegate* unmanaged[Stdcall]<nint, byte*, byte> offset);

    	[DllImport(__DllName, EntryPoint = "set_object_as_u16", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
    	internal static extern void set_object_as_u16(delegate* unmanaged[Stdcall]<nint, byte*, ushort> offset);

     	[DllImport(__DllName, EntryPoint = "set_object_as_u32", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
     	internal static extern void set_object_as_u32(delegate* unmanaged[Stdcall]<nint, byte*, uint> offset);

     	[DllImport(__DllName, EntryPoint = "set_object_as_u64", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
     	internal static extern void set_object_as_u64(delegate* unmanaged[Stdcall]<nint, byte*, ulong> offset);

     	[DllImport(__DllName, EntryPoint = "set_object_as_string", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
     	internal static extern void set_object_as_string(delegate* unmanaged[Stdcall]<nint, byte*, nint> offset);

   	    [DllImport(__DllName, EntryPoint = "set_get_object_instance", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
   	    internal static extern void set_get_object_instance(delegate* unmanaged[Stdcall]<ObjectInitializer*, nint> offset);

   	    // OnModLoading support

   	    [DllImport(__DllName, EntryPoint = "on_mod_loading", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
   	    internal static extern void on_mod_loading(nint pModConfig);
     	{{/if}}
    }
}

namespace {{mod_id}} 
{

    {{#if csharp_function_invoke}}

    // This must stay in sync with ObjectInitializer in riri_mod_tools_rt::system!
    [StructLayout(LayoutKind.Sequential)]
    public struct ObjectInitializer
    {
        internal ulong Hash;
        internal nint Size;
    }

    // This must stay in sync with StringData in riri_mod_tools_rt::system!
    [StructLayout(LayoutKind.Sequential)]
    public struct StringData
    {
        internal nint Ptr;
        internal nint Len;
    }

    // This must stay in sync with ArrayData in riri_mod_tools_rt::system!
    [StructLayout(LayoutKind.Sequential)]
    public struct ArrayData
    {
        internal ulong Hash;
        internal nint Len;
    }

    internal class MethodListInitComparer : IEqualityComparer<(ulong, MethodInfo)>
    {
        public bool Equals((ulong, MethodInfo) x, (ulong, MethodInfo) y)
            => x.Item1.Equals(y.Item1);

        public int GetHashCode([DisallowNull] (ulong, MethodInfo) obj)
            => obj.Item1.GetHashCode();
    }

    internal class MethodList
    {
        public MethodList(Type type)
        {
            HashSet<Type> DistinctTypes = [];
            ImportMethods(type);
            return;

            void ImportMethods(Type type)
            {
                if (!DistinctTypes.Add(type)) return;
                foreach (var (Hash, Method) in (type.GetMethods()
                        .Where(x => x.ReturnType.FullName != null
                            && x.GetParameters().All(y => y.ParameterType.FullName != null)
                            && !x.GetParameters().Any(y => y.IsIn || y.IsOut))
                        .Select(Method =>
                        {
                            var ParamHashes = Method.GetParameters().Select(x =>
                            {
                                var ParamType = x.ParameterType;
                                return ParamType.FullName != null ? ParamType.FullName!.ToXxh3() : ParamType.Name.ToXxh3();
                            });
                            var FinalHash = Method.Name.ToXxh3();
                            FinalHash = ParamHashes.Aggregate(FinalHash, (current, Param) => current + Param);
                            return (FinalHash, Method);
                        })))
                {
                    if (Methods.TryAdd(Hash, Method))
                        Parameters[Hash] = new();
                }
                foreach (var impl in type.GetInterfaces())
                    ImportMethods(impl);
            }
        }

        public Dictionary<ulong, MethodInfo> Methods { get; } = [];
        public Dictionary<ulong, List<nint>> Parameters { get; } = [];
    }
    {{/if}}

    public unsafe partial class Mod
    {
		[UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
		public static unsafe void ReloadedLoggerWrite(nint p, nint len, int color, int level, byte showPrefix)
		{
		    var fmt = ((showPrefix & 1) != 0) switch
		    {
		        true => $"[{{logger_prefix}}] {Marshal.PtrToStringUTF8(p, (int)len)}",
		        false => Marshal.PtrToStringUTF8(p, (int)len),
		    };
		    _logger!.WriteAsync(fmt, Color.FromArgb(color));
		}

		[UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
		public static unsafe void ReloadedLoggerWriteLine(nint p, nint len, int color, int level, byte showPrefix)
		{
			var fmt = ((showPrefix & 1) != 0) switch
        	{
        	    true => $"[{{logger_prefix}}] {Marshal.PtrToStringUTF8(p, (int)len)}",
        	    false => Marshal.PtrToStringUTF8(p, (int)len),
        	};
		    _logger!.WriteLineAsync(fmt, Color.FromArgb(color));
		}

		public void RegisterLogger() 
		{
		    {{utility_namespace}}.set_reloaded_logger(&ReloadedLoggerWrite);
		    {{utility_namespace}}.set_reloaded_logger_newline(&ReloadedLoggerWriteLine);
			var LoggerColorString = Marshal.StringToHGlobalAnsi("{{logger_color}}");
			if (!{{utility_namespace}}.set_logger_color(LoggerColorString)) {
				throw new Exception("{{logger_color}} is not a valid color name or hex code!");
			}
			Marshal.FreeHGlobal(LoggerColorString);
		}

		public void RegisterModLoaderAPI()
		{
			{{utility_namespace}}.set_get_directory_for_mod(&GetDirectoryForMod);
			{{utility_namespace}}.set_get_config_directory(&GetConfigDirectory);
			{{utility_namespace}}.set_free_csharp_string(&FreeCSharpString);
			{{utility_namespace}}.set_find_pattern(&FindPattern);

			// C# method invocation
			{{#if csharp_function_invoke}}

			HashSet<string> ExcludedAssemblies = [
			    // Could not load file or assembly 'Microsoft.Extensions.Logging.Abstractions, Version=9.0.0.0, Culture=neutral,
			    // PublicKeyToken=adb9793829ddae60'. An operation is not legal in the current state. (0x80131509)
			    "RyoTune.Reloaded",
			    // Could not GetTypes on assembly 'Reloaded.Hooks.ReloadedII.Interfaces, Version=1.14.0.0, Culture=neutral,
			    // PublicKeyToken=null' - An item with the same key has already been added. Key: 17815670155489228776
			    "Reloaded.Hooks.ReloadedII.Interfaces",
			];

			{{utility_namespace}}.set_push_parameter(&PushParameter);
			{{utility_namespace}}.set_call_function(&CallFunction);
			{{utility_namespace}}.set_free_object(&FreeObject);
			{{utility_namespace}}.set_object_as_u8(&ObjectAsU8);
			{{utility_namespace}}.set_object_as_u16(&ObjectAsU16);
			{{utility_namespace}}.set_object_as_u32(&ObjectAsU32);
			{{utility_namespace}}.set_object_as_u64(&ObjectAsU64);
			{{utility_namespace}}.set_object_as_string(&ObjectAsString);
			{{utility_namespace}}.set_get_object_instance(&GetObjectInstance);

			foreach (var BasicType in BasicTypeGenerators)
			    BasicTypes.Add(CreateTypePath(BasicType, "riri_mod_tools_rt").ToXxh3(), BasicType);

			foreach (var (R2Type, GetSingleton) in Reloaded2Interfaces)
			{
			    BasicTypes.Add(CreateTypePath(R2Type, "riri_mod_tools_rt").ToXxh3(), R2Type);
			    SingletonTypes.Add(R2Type, GetSingleton);
			}

            foreach (var Assembly in AppDomain.CurrentDomain.GetAssemblies()
                         .Where(x => x.FullName!.Split(".")[0] != "System"
                            && !ExcludedAssemblies.Contains(x.GetName().Name!)))
            {
                try
                {
                    foreach (var Type in Assembly.GetTypes()
                                 .Where(x => x is { IsInterface: true, IsGenericType: false, FullName: not null }))
                    {
                        var Namespace = Reloaded2Interfaces.ContainsKey(Type) ? "riri_mod_tools_rt" : null;
                        Types.TryAdd(CreateTypePath(Type, Namespace).ToXxh3(), new(Type));
                    }
                }
                catch (Exception ex)
                {
                    _logger!.WriteLineAsync($"[{{logger_prefix}}]: Could not GetTypes on assembly '{Assembly.FullName!}' - {ex.Message}", Color.Red);
                }
            }

            // To call System.Array methods when converting into Vec<T>
            Types.TryAdd(CreateTypePath(typeof(IEqualityComparer), null).ToXxh3(), new(typeof(IEqualityComparer)));
            Types.TryAdd(CreateTypePath(typeof(Array), null).ToXxh3(), new(typeof(Array)));
            {{/if}}

            _logger!.OnWrite += (_, p) => {{utility_namespace}}.on_reloaded_logger(Marshal.StringToHGlobalUni(p.text));
            _logger!.OnWriteLine += (_, p) => {{utility_namespace}}.on_reloaded_logger_newline(Marshal.StringToHGlobalUni(p.text));
		}

		[UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
		public static unsafe nint GetDirectoryForMod()
		{
			var ModDirectory = _modLoader!.GetDirectoryForModId(_modConfig!.ModId);
			return Marshal.StringToHGlobalUni(ModDirectory);
		}

        [UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
    	public static unsafe nint GetConfigDirectory()
    	{
    		var ModDirectory = _modLoader!.GetModConfigDirectory(_modConfig!.ModId);
    		return Marshal.StringToHGlobalUni(ModDirectory);
    	}

		[UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
		public static unsafe void FreeCSharpString(nint p)
		{
			Marshal.FreeHGlobal(p);
		}

		[UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
		public static unsafe nint FindPattern(nint pat, nint start, int len)
		{
			var scanner = new Scanner((byte*)start, len);
			var result = scanner.FindPattern(Marshal.PtrToStringUTF8(pat));
			if (result.Found)
			{
				return start + result.Offset;
			} else
			{
				return nint.Zero;
			}
		}

        {{#if csharp_function_invoke}}

    	private static List<Type> BasicTypeGenerators =
    	[
           typeof(byte), typeof(sbyte), typeof(bool), typeof(short), typeof(ushort), typeof(int), typeof(uint),
           typeof(long), typeof(ulong), typeof(float), typeof(double), typeof(string), typeof(Array)
    	];
    	private static Dictionary<Type, Func<object>> Reloaded2Interfaces = new()
    	{
    	    { typeof(IModConfig), () => _modConfig! }
    	};

        // All reflected types
		private static ConcurrentDictionary<ulong, MethodList> Types = [];

		// For object initialization: basic types
		private static Dictionary<ulong, Type> BasicTypes = [];
		// Singleton objects - these correspond to Reloaded-II interfaces from other mods
        private static Dictionary<Type, Func<object>> SingletonTypes = [];

        #nullable enable
        private static string CreateTypePath(Type type, string? overrideNamespace = null)
        {
            var PathArr = type.FullName!.Split("`", 2); // Remove generic arguments from full name
            var GenericFmt = PathArr.Length > 1 ? $"<{string.Join(", ", type.GenericTypeArguments.Select(x => CreateTypePath(x)))}>" : string.Empty;
            var Path = type.IsArray ? PathArr[0][..^2] : PathArr[0];
            var Parts = Path.Split(".");
            var RustName = string.Join("::", Parts.Select((x, i) => i != Parts.Length - 1 ? x.ToSnakeCase() : x));
            RustName = $"{overrideNamespace ?? "crate"}::{RustName}{GenericFmt}";
            if (type.IsArray) return $"crate::system::Array<{RustName}>";
            return RustName;
        }
        #nullable disable

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
        public static void PushParameter(ulong TypeHash, ulong MethodHash, nint pParam)
            => Types[TypeHash].Parameters[MethodHash].Add(pParam);

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
        public static nint CallFunction(ulong TypeHash, ulong MethodHash, nint pObject)
        {
            var Object = GCHandle.FromIntPtr(pObject).Target;
            var Methods = Types[TypeHash];
            var ResultObj = Methods.Methods[MethodHash]
                .Invoke(Object, Methods.Parameters[MethodHash]
                    .Select(x => GCHandle.FromIntPtr(x).Target!).ToArray());
            Methods.Parameters[MethodHash].Clear();
            return GCHandle.ToIntPtr(GCHandle.Alloc(ResultObj));
        }

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
        public static void FreeObject(nint pObject)
            => GCHandle.FromIntPtr(pObject).Free();

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
        public static byte ObjectAsU8(nint pObject, byte* pSuccess)
        {
            var Object = GCHandle.FromIntPtr(pObject).Target;
            if (Object.GetType() == typeof(byte))
                return (byte)Object;
            if (Object.GetType() == typeof(sbyte))
                return (byte)(sbyte)Object;
            if (Object.GetType() == typeof(bool))
                return ((bool)Object).ToByte();
            *pSuccess = 0;
            return 0;
        }

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
        public static ushort ObjectAsU16(nint pObject, byte* pSuccess)
        {
            var Object = GCHandle.FromIntPtr(pObject).Target;
            if (Object.GetType() == typeof(ushort))
                return (ushort)Object;
            if (Object.GetType() == typeof(short))
                return (ushort)(short)Object;
            *pSuccess = 0;
            return 0;
        }

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
        public static uint ObjectAsU32(nint pObject, byte* pSuccess)
        {
            var Object = GCHandle.FromIntPtr(pObject).Target;
            if (Object.GetType() == typeof(uint))
                return (uint)Object;
            if (Object.GetType() == typeof(int))
                return (uint)(int)Object;
            if (Object.GetType() == typeof(float))
                return BitConverter.SingleToUInt32Bits((float)Object);
            *pSuccess = 0;
            return 0;
        }

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
        public static ulong ObjectAsU64(nint pObject, byte* pSuccess)
        {
            var Object = GCHandle.FromIntPtr(pObject).Target;
            if (Object.GetType() == typeof(ulong))
                return (ulong)Object;
            if (Object.GetType() == typeof(long))
                return (ulong)(long)Object;
            if (Object.GetType() == typeof(double))
                return BitConverter.DoubleToUInt64Bits((double)Object);
            *pSuccess = 0;
            return 0;
        }

        [UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
        public static nint ObjectAsString(nint pObject, byte* pSuccess)
        {
            var Object = GCHandle.FromIntPtr(pObject).Target;
            if (Object.GetType() == typeof(string))
                return Marshal.StringToHGlobalUni((string)Object);
            *pSuccess = 0;
            return nint.Zero;
        }

        private static nint GetObjectFromType<T>(ObjectInitializer* initializer) where T : unmanaged
            => GCHandle.ToIntPtr(GCHandle.Alloc(*(T*)(initializer + 1), GCHandleType.Pinned));

        [UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
        public static nint GetObjectInstance(ObjectInitializer* initializer)
        {
            if (!BasicTypes.TryGetValue(initializer->Hash, out var Type)) return nint.Zero;
            if (SingletonTypes.TryGetValue(Type, out var Callback))
                return GCHandle.ToIntPtr(GCHandle.Alloc(Callback()));
            if (Type == typeof(byte))
                return GetObjectFromType<byte>(initializer);
            if (Type == typeof(sbyte))
                return GetObjectFromType<sbyte>(initializer);
            if (Type == typeof(bool))
                return GetObjectFromType<bool>(initializer);
            if (Type == typeof(short))
                return GetObjectFromType<short>(initializer);
            if (Type == typeof(ushort))
                return GetObjectFromType<ushort>(initializer);
            if (Type == typeof(int))
                return GetObjectFromType<int>(initializer);
            if (Type == typeof(uint))
                return GetObjectFromType<uint>(initializer);
            if (Type == typeof(long))
                return GetObjectFromType<long>(initializer);
            if (Type == typeof(ulong))
                return GetObjectFromType<ulong>(initializer);
            if (Type == typeof(float))
                return GetObjectFromType<float>(initializer);
            if (Type == typeof(double))
                return GetObjectFromType<double>(initializer);
            if (Type == typeof(string))
            {
                var stringData = (StringData*)(initializer + 1);
                return GCHandle.ToIntPtr(GCHandle.Alloc(Marshal.PtrToStringUTF8(stringData->Ptr, (int)stringData->Len)));
            }
            if (Type == typeof(Array))
            {
                var arrayData = (ArrayData*)(initializer + 1);
                if (!BasicTypes.TryGetValue(arrayData->Hash, out var ValueType)) return nint.Zero;
                return GCHandle.ToIntPtr(GCHandle.Alloc(Array.CreateInstance(ValueType, (int)arrayData->Len)));
            }
            return nint.Zero;
        }
        {{/if}}
        {{#if cached_signatures}}

        #nullable enable

        private static byte[]? CachedSignature = null;

        private static BinaryWriter? RegenWriter = null;

        internal struct RegenSigEntry(ulong pattern, ulong offset)
        {
            internal ulong Pattern = pattern;
            internal ulong Offset = offset;
        }

        private static List<RegenSigEntry> RegenSigs = new();

        // private static List<Action<nuint>> CacheSigCallbacks = new();
        private static Action CacheSigCallbacks = () => { };

        #nullable disable

        private static bool TryCheckSignatureCache()
        {
            // signature_cache binary format:
            // All strings are xxh3 hashed
            // u64 ExecutableHash
            // u32 Version
            // u32 ModCount
            // ModData[ModCount]:
            //      u64 ModId;
            //      u64 ModVer;
            // u64 SignatureCount
            // SigData[SignatureCount]
            //      u64 SigHash;
            //      u64 Address;
            try
            {
                var CachePath = Path.Join(_modLoader!.GetDirectoryForModId(_modConfig!.ModId), "signature_cache");
                using (var Reader = new BinaryReader(new FileStream(CachePath, FileMode.Open, FileAccess.Read)))
                {
                    CachedSignature = new byte[Reader.BaseStream.Length];
                    Reader.Read(CachedSignature, 0, (int)Reader.BaseStream.Length);
                }
                using (var Reader = new BinaryReader(new MemoryStream(CachedSignature)))
                {
                    _ = Reader.ReadUInt64(); // Executable hash
                    _ = Reader.ReadUInt32(); // Version
                    var CurrModList = _modLoader!.GetActiveMods().Select(tup => tup.Generic).ToList();
                    var CacheModCount = (int)Reader.ReadUInt32();
                    if (
                        CurrModList.Count != CacheModCount ||
                        CurrModList.Any(Mod => Mod.ModId.ToXxh3() != Reader.ReadUInt64() || Mod.ModVersion.ToXxh3() != Reader.ReadUInt64()))
                        return false;
                }
            }
            catch (FileNotFoundException)
            {
                _logger!.WriteLineAsync("{{mod_name}}: Signature cache does not exist yet, generating...");
                return false;
            }
            catch (IOException ex)
            {
                _logger!.WriteLineAsync($"{{mod_name}}: An error occurred while reading signature_cache: '{ex.Message}', regenerating...");
                return false;
            }

            return true;
        }

        private static void CheckSignatureCache()
        {
            if (TryCheckSignatureCache()) return;
            // Have to regenerate
            CachedSignature = null;
            var CachePath = Path.Join(_modLoader!.GetDirectoryForModId(_modConfig!.ModId), "signature_cache");
            RegenWriter = new BinaryWriter(new FileStream(CachePath, FileMode.Create, FileAccess.Write));
            var CurrModList = _modLoader!.GetActiveMods().Select(tup => tup.Generic).ToList();
            RegenWriter.Write({{utility_namespace}}.get_executable_hash());
            RegenWriter.Write(0);
            RegenWriter.Write((uint)CurrModList.Count);
            foreach (var Mod in CurrModList)
            {
                RegenWriter.Write(Mod.ModId.ToXxh3());
                RegenWriter.Write(Mod.ModVersion.ToXxh3());
            }
        }

        private static void FinishSignatureCache()
        {
            CacheSigCallbacks();
            if (RegenWriter != null)
            {
                RegenWriter.Write((ulong)RegenSigs.Count);
                foreach (var Entry in RegenSigs)
                {
                    RegenWriter.Write(Entry.Pattern);
                    RegenWriter.Write(Entry.Offset);
                }
                RegenWriter.Flush();
                RegenWriter.Close();
                RegenWriter = null;
            }
        }

        {{/if}}
    }
    public static class TypeExtensions
    {
        public static ulong ToXxh3(this string Value)
            => XxHash3.HashToUInt64(Encoding.UTF8.GetBytes(Value));
    }
}