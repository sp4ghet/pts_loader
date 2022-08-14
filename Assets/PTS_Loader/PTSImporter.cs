using UnityEngine;
using UnityEngine.Rendering;
using UnityEditor;
#if UNITY_2020_2_OR_NEWER
using UnityEditor.AssetImporters;
#else
using UnityEditor.Experimental.AssetImporters;
#endif

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;

namespace sp4ghet
{
    static class Loader
    {
        [StructLayout(LayoutKind.Sequential)]
        public struct Vec3<N>
        {
            public N x;
            public N y;
            public N z;
        };

        [StructLayout(LayoutKind.Sequential)]
        public struct PtsPoint
        {
            public Vec3<float> point;
            public int intensity;
            public Vec3<byte> rgb;
        };


#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
        [DllImport("__Internal", CallingConvention = CallingConvention.Cdecl)]
#else
        [DllImport("pts_loader", CallingConvention = CallingConvention.Cdecl)]
#endif
        static extern void load_from_file(string path, [Out] out IntPtr points, out uint len);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
        [DllImport("__Internal")]
#else
        [DllImport("pts_loader")]
#endif
        static extern void free_pts(IntPtr ptr, uint len);

        public static Mesh LoadFromPath(string path, Matrix4x4 offset)
        {

            IntPtr ptr = IntPtr.Zero;
            uint len;
            load_from_file(path, out ptr, out len);
            Vector3[] vertices = new Vector3[len];
            Color[] colors = new Color[len];
            var sizeOfPtsPoint = Marshal.SizeOf<PtsPoint>();

            for (int i = 0; i < len; i++)
            {
                var point = (PtsPoint)Marshal.PtrToStructure(ptr + i * sizeOfPtsPoint, typeof(PtsPoint));
                vertices[i] = offset.MultiplyPoint3x4(new Vector3(point.point.x, point.point.y, point.point.z));
                colors[i] = new Color(point.rgb.x / 255f, point.rgb.y / 255f, point.rgb.z / 255f);
            }
            free_pts(ptr, len);

            Mesh mesh = new Mesh();
            mesh.indexFormat = len > 65535 ? IndexFormat.UInt32 : IndexFormat.UInt16;
            mesh.name = Path.GetFileNameWithoutExtension(path);
            mesh.SetVertices(vertices);
            mesh.SetColors(colors);
            mesh.SetIndices(
                        Enumerable.Range(0, (int)len).ToArray(),
                        MeshTopology.Points, 0
                    );
            mesh.UploadMeshData(true);
            return mesh;
        }
    }

    [ScriptedImporter(1, "pts")]
    public class PTSImporter : ScriptedImporter
    {

        public enum ContainerType { Mesh, ComputeBuffer, Texture }

        [SerializeField] Vector3 position;
        [SerializeField] Vector3 rotation;
        [SerializeField] Vector3 scale = Vector3.one;

        [SerializeField] Material mat;


        public override void OnImportAsset(AssetImportContext ctx)
        {
            var gameObject = new GameObject();
            Quaternion rot = Quaternion.Euler(rotation);
            Matrix4x4 trs = Matrix4x4.TRS(position, rot, scale);
            var path = ctx.assetPath;
            var mesh = Loader.LoadFromPath(path, trs);

            var meshFilter = gameObject.AddComponent<MeshFilter>();
            meshFilter.sharedMesh = mesh;

            var meshRenderer = gameObject.AddComponent<MeshRenderer>();
            meshRenderer.sharedMaterial = mat;

            ctx.AddObjectToAsset("prefab", gameObject);
            if (mesh != null) ctx.AddObjectToAsset("mesh", mesh);

            ctx.SetMainObject(gameObject);
        }

    }
}
