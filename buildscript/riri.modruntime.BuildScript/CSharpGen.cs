using System.Reflection;
using Minerals.StringCases;
using Reloaded.Mod.Interfaces;

namespace riri.modruntime.BuildScript;

public class CSharpGen
{
     private static HashSet<Type> HasBuiltins =
    [
        typeof(byte),
        typeof(sbyte),
        typeof(char),
        typeof(bool),
        typeof(short),
        typeof(ushort),
        typeof(int),
        typeof(uint),
        typeof(long),
        typeof(ulong),
        typeof(float),
        typeof(double),
        typeof(string),
        typeof(void),
    ];
 
    private static HashSet<Type> ReloadedInterfaces =
    [
        typeof(IModConfig),
    ];
    
    internal class MethodList
    {
        public MethodList(Type type)
        {
            List<(ulong, MethodInfo)> MethodList = new();
            HashSet<Type> DistinctTypes = [];
            ImportMethods(type);
            Methods = MethodList.ToDictionary();
            Parameters = MethodList.Select(x => (x.Item1, new List<nint>())).ToDictionary();
            return;

            void ImportMethods(Type type)
            {
                if (!DistinctTypes.Add(type)) return;
                MethodList.AddRange(type.GetMethods()
                    .Select(Method =>
                    {
                        var ParamHashes = Method.GetParameters().Select(x =>
                        {
                            var ParamType = x.ParameterType;
                            return ParamType.FullName != null ? ParamType.FullName!.ToXxh3() : 0;
                        });
                        var FinalHash = Method.Name.ToXxh3();
                        FinalHash = ParamHashes.Aggregate(FinalHash, (current, Param) => current + Param);
                        return (FinalHash, Method);
                    }));
                foreach (var impl in type.GetInterfaces())
                    ImportMethods(impl);
            }
        }

        public Dictionary<ulong, MethodInfo> Methods { get; }
        public Dictionary<ulong, List<nint>> Parameters { get; } = [];
    }

    
    // TODO: Automatically write into riri-mod-tools-rt
    // public static void Main(string[] Args)
    // {
    //     Bindgen([Assembly.Load("Reloaded.Mod.Interfaces")]);
    // }

    private static void Bindgen(Assembly[] Assemblies)
    {
        Dictionary<Type, string> TypesToGenerate = [];
        Dictionary<Type, List<MethodInfo>> MethodsToGenerate = [];

        foreach (var Assembly in AppDomain.CurrentDomain.GetAssemblies()
                     .Where(x => x.FullName!.Split(".")[0] != "System"))
        {
            foreach (var Type in Assembly.GetTypes().Where(x => x is { IsInterface: true, FullName: not null }))
            {
                AddType(Type);
                HashSet<string> PropertyMethods = [];
                foreach (var Property in Type.GetProperties())
                {
                    if (Property.GetMethod != null)
                    {
                        PropertyMethods.Add(Property.GetMethod.Name);
                        AddType(Property.GetMethod.ReturnType);
                        AddMethod(Type, Property.GetMethod);
                    }
                    if (Property.SetMethod != null)
                    {
                        PropertyMethods.Add(Property.SetMethod.Name);
                        AddType(Property.SetMethod.GetParameters()[0].ParameterType);
                        AddMethod(Type, Property.SetMethod);
                    }
                }
                foreach (var Method in Type.GetMethods()
                             .Where(x => x.ReturnType.FullName != null 
                                         && x.GetParameters().All(y => y.ParameterType.FullName != null) 
                                         && !PropertyMethods.Contains(x.Name)
                                         && !x.GetParameters().Any(y => y.IsIn || y.IsOut)))
                {
                    var ReturnType = Method.ReturnType;
                    AddType(ReturnType);
                    foreach (var Param in Method.GetParameters())
                        AddType(Param.ParameterType);
                    AddMethod(Type, Method);
                }
            }
        }

        foreach (var (Type, Name) in TypesToGenerate)
        {
            if (Type.IsGenericType || Type.IsArray || HasBuiltins.Contains(Type)) continue;
            var Namespace = ReloadedInterfaces.Contains(Type) ? "riri_mod_tools_rt" : null;
            List<string> LinesForType =
            [
                "#[derive(Debug)]",
                $"pub struct {Type.Name}(usize);",
                string.Empty,
                $"impl ObjectHash for {Type.Name} {{",
                $"\tconst HASH: u64 = 0x{CreateTypePath(Type, Namespace).ToXxh3():x};",
                "}",
                string.Empty,
                $"impl ObjectInitializable for {Type.Name} {{",
                "\ttype InitType = ();",
                "\tfn new(_: Self::InitType) -> Result<Self, crate::interop::InteropError> {",
                "\t\tlet handle = crate::interop::ObjectInitHandle::<()>::new(Self::HASH, ());",
                "\t\tlet result = unsafe { crate::interop::get_object_instance(handle.raw()) };",
                "\t\tmatch result {",
                "\t\t\t0 => Err(crate::interop::InteropError::CouldNotMakeObjectInstance),",
                "\t\t\t_ => Ok(Self(Object(result)))",
                "\t\t}",
                "\t}",
                "}",
                string.Empty,
                $"impl {Type.Name} {{"
            ];

            HashSet<Type> GeneratedDefinitions = [];
            
            BuildMethodImpl(Type);
            
            void BuildMethodImpl(Type CurrentType)
            {
                if (!GeneratedDefinitions.Contains(CurrentType) 
                    && MethodsToGenerate.TryGetValue(CurrentType, out var Methods))
                {
                    GeneratedDefinitions.Add(CurrentType);
                    LinesForType.Add($"\t// impl {CurrentType.Name}");
                    foreach (var Method in Methods)
                    {
                        var ParamsFmt = string.Join(", ", Method.GetParameters()
                            .Select(x => $"{x.Name!.ToSnakeCase()}: &{TypesToGenerate[x.ParameterType]}"));
                        ParamsFmt = ParamsFmt == string.Empty ? string.Empty : $", {ParamsFmt}";
                        var ReturnType = Method.ReturnType == typeof(void) ? string.Empty : $" -> {TypesToGenerate[Method.ReturnType]}";
                        LinesForType.Add($"\tpub fn {Method.Name.ToSnakeCase()}(&self{ParamsFmt}){ReturnType} {{");
                        var MethodHash = Method.Name.ToXxh3();
                        foreach (var Param in Method.GetParameters())
                            LinesForType.Add($"\t\triri_mod_tools_rt::interop::push_parameter(Self::HASH, 0x{MethodHash:x}, **{Param.Name!.ToSnakeCase()});");
                        var ReturnStmt = Method.ReturnType == typeof(void) ? ";" : string.Empty;
                        var ReturnFmt = $"riri_mod_tools_rt::interop::call_function(Self::HASH, 0x{MethodHash:x}){ReturnStmt}";
                        if (Method.ReturnType != typeof(void))
                        {
                            var ReturnNamespace = ReloadedInterfaces.Contains(Method.ReturnType) ? "riri_mod_tools_rt" : null;
                            var (ReturnPath, ObjectPath) = (CreateTypePath(Method.ReturnType, ReturnNamespace),
                                CreateTypePath(typeof(Object), ReturnNamespace));
                            ReturnFmt = $"unsafe {{ {ReturnPath}::new_unchecked({ObjectPath}::new_unchecked({ReturnFmt})) }}";
                        }
                        LinesForType.Add($"\t\t{ReturnFmt}");
                        LinesForType.Add("\t}");
                    }
                }
                foreach (var Impl in CurrentType.GetInterfaces())
                    BuildMethodImpl(Impl);
            }
            
            LinesForType.Add("}");
            foreach (var Line in LinesForType)
                Console.WriteLine(Line);
        }
        return;

        void AddType(Type type)
        {
            if (!TypesToGenerate.ContainsKey(type))
            {
                var Namespace = ReloadedInterfaces.Contains(type) ? "riri_mod_tools_rt" : null;
                TypesToGenerate.TryAdd(type, CreateTypePath(type, Namespace)); 
            }
            foreach (var Arg in type.GenericTypeArguments)
                AddType(Arg);
        }
        
        void AddMethod(Type type, MethodInfo method)
        {
            MethodsToGenerate.TryAdd(type, []);
            MethodsToGenerate[type].Add(method);
        };
    }

    private static string CreateTypePath(Type type, /*bool bConvertBasicTypes = true,*/ string? overrideNamespace = null)
    {
        if (HasBuiltins.Contains(type)) overrideNamespace = "riri_mod_tools_rt";
        // var bIsBasicType = DirectlyUsableTypes.TryGetValue(type, out var RustType);
        // if (bConvertBasicTypes && bIsBasicType) return RustType;
        var PathArr = type.FullName!.Split("`", 2); // Remove generic arguments from full name
        var GenericFmt = PathArr.Length > 1 ? $"<{string.Join(", ", type.GenericTypeArguments.Select(x => CreateTypePath(x)))}>" : string.Empty;
        var Path = type.IsArray ? PathArr[0][..^2] : PathArr[0];
        var Parts = Path.Split(".");
        var RustName = string.Join("::", Parts.Select((x, i) => i != Parts.Length - 1 ? x.ToSnakeCase() : x));
        RustName = $"{overrideNamespace ?? "crate"}::{RustName}{GenericFmt}";
        if (type.IsArray) return $"riri_mod_tools_rt::system::Array<{RustName}>";
        return RustName;
    }   
}